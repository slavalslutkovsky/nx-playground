//! Integration tests for the email library

use email::models::{Email, EmailEvent, EmailPriority};
use email::provider::{EmailProvider, MockSmtpProvider};
use email::stream::{EmailConsumer, EmailProducer};
use email::templates::{EmailTemplate, InMemoryTemplateStore, TemplateStore};
use redis::aio::ConnectionManager;
use serde_json::json;

/// Helper to create a Redis ConnectionManager from TestRedis
async fn create_connection_manager(connection_string: &str) -> ConnectionManager {
    let client = redis::Client::open(connection_string).expect("Failed to create Redis client");
    ConnectionManager::new(client)
        .await
        .expect("Failed to create ConnectionManager")
}

mod stream_tests {
    use super::*;
    use test_utils::TestRedis;

    #[tokio::test]
    async fn test_producer_consumer_flow() {
        let redis = TestRedis::new().await;
        let conn_manager = create_connection_manager(redis.connection_string()).await;

        // Create a producer and send email
        let producer = EmailProducer::new(conn_manager.clone());

        let email = Email::new("test@example.com", "Test Subject")
            .with_text("Test body content")
            .with_priority(EmailPriority::High);

        let email_id = email.id.clone();
        let stream_id = producer.send(email).await.expect("Failed to send email");

        assert!(!stream_id.is_empty());

        // Create consumer and read email
        let consumer = EmailConsumer::new(conn_manager.clone(), "test-consumer");
        consumer
            .init_consumer_group()
            .await
            .expect("Failed to init consumer group");

        let emails = consumer
            .read_emails(10, 1000)
            .await
            .expect("Failed to read emails");

        assert_eq!(emails.len(), 1);

        let (received_stream_id, event) = &emails[0];
        assert_eq!(received_stream_id, &stream_id);

        if let EmailEvent::SendEmail(received_email) = event {
            assert_eq!(received_email.id, email_id);
            assert_eq!(received_email.to, "test@example.com");
            assert_eq!(received_email.subject, "Test Subject");
            assert_eq!(received_email.priority, EmailPriority::High);
        } else {
            panic!("Expected SendEmail event");
        }

        // Acknowledge the message
        consumer
            .ack(&stream_id)
            .await
            .expect("Failed to ack message");
    }

    #[tokio::test]
    async fn test_consumer_group_idempotent() {
        let redis = TestRedis::new().await;
        let conn_manager = create_connection_manager(redis.connection_string()).await;

        let consumer = EmailConsumer::new(conn_manager, "test-consumer-idempotent");

        // Initialize consumer group multiple times should not fail
        consumer.init_consumer_group().await.unwrap();
        consumer.init_consumer_group().await.unwrap();
        consumer.init_consumer_group().await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_emails() {
        let redis = TestRedis::new().await;
        let conn_manager = create_connection_manager(redis.connection_string()).await;

        let producer = EmailProducer::new(conn_manager.clone());

        // Send multiple emails
        for i in 0..3 {
            let email = Email::new(format!("user{}@example.com", i), format!("Subject {}", i))
                .with_text(format!("Body {}", i));
            producer.send(email).await.unwrap();
        }

        let consumer = EmailConsumer::new(conn_manager, "test-consumer-multi");
        consumer.init_consumer_group().await.unwrap();

        let emails = consumer.read_emails(10, 1000).await.unwrap();
        assert_eq!(emails.len(), 3);
    }
}

mod template_tests {
    use super::*;

    #[tokio::test]
    async fn test_template_rendering() {
        let store = InMemoryTemplateStore::new();

        // Add a custom template
        let template = EmailTemplate {
            name: "test_template".to_string(),
            subject: "Hello {{name}}!".to_string(),
            body_text: Some("Dear {{name}}, your order #{{order_id}} is ready.".to_string()),
            body_html: Some("<h1>Hello {{name}}!</h1><p>Order #{{order_id}} ready.</p>".to_string()),
        };

        store.set(template).await.unwrap();

        // Retrieve and render
        let template = store.get("test_template").await.unwrap().unwrap();
        let data = json!({
            "name": "John",
            "order_id": "12345"
        });

        let rendered = template.render(&data).unwrap();

        assert_eq!(rendered.subject, "Hello John!");
        assert_eq!(
            rendered.body_text.unwrap(),
            "Dear John, your order #12345 is ready."
        );
        assert!(rendered.body_html.unwrap().contains("Hello John!"));
    }

    #[tokio::test]
    async fn test_default_templates() {
        let store = InMemoryTemplateStore::with_defaults();

        // Templates are now initialized synchronously, no need to wait

        // Check welcome template exists
        let welcome = store.get("welcome").await.unwrap();
        assert!(welcome.is_some());

        let welcome = welcome.unwrap();
        assert!(welcome.subject.contains("{{app_name}}"));

        // Check password_reset template exists
        let reset = store.get("password_reset").await.unwrap();
        assert!(reset.is_some());
    }

    #[tokio::test]
    async fn test_template_list() {
        let store = InMemoryTemplateStore::new();

        store
            .set(EmailTemplate {
                name: "template1".to_string(),
                subject: "Subject 1".to_string(),
                body_text: None,
                body_html: None,
            })
            .await
            .unwrap();

        store
            .set(EmailTemplate {
                name: "template2".to_string(),
                subject: "Subject 2".to_string(),
                body_text: None,
                body_html: None,
            })
            .await
            .unwrap();

        let templates = store.list().await.unwrap();
        assert_eq!(templates.len(), 2);
        assert!(templates.contains(&"template1".to_string()));
        assert!(templates.contains(&"template2".to_string()));
    }
}

mod mock_provider_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_captures_emails() {
        let provider = MockSmtpProvider::new();

        let email1 = Email::new("user1@example.com", "Subject 1").with_text("Body 1");
        let email2 = Email::new("user2@example.com", "Subject 2").with_html("<p>Body 2</p>");

        provider.send(&email1).await.unwrap();
        provider.send(&email2).await.unwrap();

        assert_eq!(provider.sent_count().await, 2);

        let sent = provider.sent_emails().await;
        assert_eq!(sent[0].to, "user1@example.com");
        assert_eq!(sent[1].to, "user2@example.com");
    }

    #[tokio::test]
    async fn test_mock_provider_health_check() {
        let provider = MockSmtpProvider::new();
        assert!(provider.health_check().await.is_ok());

        let failing_provider = MockSmtpProvider::failing("Down for maintenance");
        assert!(failing_provider.health_check().await.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_clear() {
        let provider = MockSmtpProvider::new();

        let email = Email::new("test@example.com", "Test").with_text("Body");
        provider.send(&email).await.unwrap();

        assert_eq!(provider.sent_count().await, 1);

        provider.clear().await;

        assert_eq!(provider.sent_count().await, 0);
    }
}

mod email_model_tests {
    use super::*;

    #[test]
    fn test_email_builder() {
        let email = Email::new("recipient@example.com", "Test Subject")
            .with_text("Plain text body")
            .with_html("<p>HTML body</p>")
            .with_priority(EmailPriority::High)
            .with_template("welcome", json!({"name": "John"}));

        assert_eq!(email.to, "recipient@example.com");
        assert_eq!(email.subject, "Test Subject");
        assert_eq!(email.body_text, Some("Plain text body".to_string()));
        assert_eq!(email.body_html, Some("<p>HTML body</p>".to_string()));
        assert_eq!(email.priority, EmailPriority::High);
        assert_eq!(email.template, Some("welcome".to_string()));
    }

    #[test]
    fn test_email_retry() {
        let mut email = Email::new("test@example.com", "Test");

        assert!(email.can_retry());
        assert_eq!(email.retry_count, 0);

        email.increment_retry();
        assert!(email.can_retry());
        assert_eq!(email.retry_count, 1);

        email.increment_retry();
        email.increment_retry();
        assert!(!email.can_retry()); // Default max_retries is 3
    }

    #[test]
    fn test_email_serialization() {
        let email = Email::new("test@example.com", "Test Subject")
            .with_text("Body")
            .with_priority(EmailPriority::Low);

        let json = serde_json::to_string(&email).unwrap();
        let deserialized: Email = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.to, email.to);
        assert_eq!(deserialized.subject, email.subject);
        assert_eq!(deserialized.priority, email.priority);
    }
}
