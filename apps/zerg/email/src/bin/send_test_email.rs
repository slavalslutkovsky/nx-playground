//! Test binary to send a test email via Redis Streams
//!
//! Run with: cargo run -p zerg_email --bin send_test_email

use email::{Email, EmailPriority, EmailProducer};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Get Redis host from env
    let redis_url = std::env::var("REDIS_HOST")
        .or_else(|_| std::env::var("REDIS_URL"))
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    println!("Connecting to Redis at {}", redis_url);

    // Connect to Redis directly (simple connection, no retry wrapper)
    let client = redis::Client::open(redis_url.as_str())?;
    let redis = redis::aio::ConnectionManager::new(client).await?;

    // Test PING
    let mut conn = redis.clone();
    let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
    println!("Redis PING: {}", pong);

    let producer = EmailProducer::new(redis);

    // Create a test email WITHOUT template (direct text/html)
    let email = Email::new("test@example.com", "Test Email from Zerg")
        .with_text("Hello! This is a test email sent via Redis Streams.")
        .with_html("<h1>Hello!</h1><p>This is a test email sent via Redis Streams.</p>")
        .with_priority(EmailPriority::High);

    println!("Sending test email to: {}", email.to);
    println!("Subject: {}", email.subject);

    // Send it to stream
    let stream_id = producer.send(email).await?;

    println!("Email queued successfully!");
    println!("Stream ID: {}", stream_id);

    Ok(())
}
