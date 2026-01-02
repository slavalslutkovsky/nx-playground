//! NATS implementation of the MessageBroker trait

use super::{EventEnvelope, MessageBroker, MessageStream, ReceivedMessage};
use async_nats::{Client, Subscriber};
use async_trait::async_trait;
use eyre::{Result, WrapErr};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, instrument};

/// NATS-based message broker implementation
pub struct NatsBroker {
    client: Client,
}

impl NatsBroker {
    /// Connect to NATS server
    pub async fn connect(url: &str) -> Result<Self> {
        let client = async_nats::connect(url)
            .await
            .wrap_err_with(|| format!("Failed to connect to NATS at {}", url))?;

        Ok(Self { client })
    }

    /// Connect with custom options
    pub async fn connect_with_options(url: &str, name: &str) -> Result<Self> {
        let client = async_nats::ConnectOptions::new()
            .name(name)
            .connect(url)
            .await
            .wrap_err_with(|| format!("Failed to connect to NATS at {}", url))?;

        Ok(Self { client })
    }

    /// Get the underlying NATS client for advanced operations
    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[async_trait]
impl MessageBroker for NatsBroker {
    #[instrument(skip(self, event), fields(subject = %subject))]
    async fn publish<T: Serialize + Send + Sync>(
        &self,
        subject: &str,
        event: &EventEnvelope<T>,
    ) -> Result<()> {
        let payload = serde_json::to_vec(event)?;
        self.client
            .publish(subject.to_string(), payload.into())
            .await
            .wrap_err("Failed to publish message")?;

        debug!(event_id = %event.id, "Published event");
        Ok(())
    }

    async fn publish_raw(&self, subject: &str, payload: &[u8]) -> Result<()> {
        self.client
            .publish(subject.to_string(), payload.to_vec().into())
            .await
            .wrap_err("Failed to publish raw message")?;
        Ok(())
    }

    async fn subscribe(&self, subject: &str) -> Result<Box<dyn MessageStream>> {
        let subscriber = self
            .client
            .subscribe(subject.to_string())
            .await
            .wrap_err_with(|| format!("Failed to subscribe to {}", subject))?;

        Ok(Box::new(NatsMessageStream { subscriber }))
    }

    #[instrument(skip(self, request), fields(subject = %subject))]
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        subject: &str,
        request: &T,
    ) -> Result<R> {
        let payload = serde_json::to_vec(request)?;
        let response = self
            .client
            .request(subject.to_string(), payload.into())
            .await
            .wrap_err("Request failed")?;

        let result: R = serde_json::from_slice(&response.payload)?;
        Ok(result)
    }

    async fn queue_subscribe(
        &self,
        subject: &str,
        queue_group: &str,
    ) -> Result<Box<dyn MessageStream>> {
        let subscriber = self
            .client
            .queue_subscribe(subject.to_string(), queue_group.to_string())
            .await
            .wrap_err_with(|| format!("Failed to queue subscribe to {}", subject))?;

        Ok(Box::new(NatsMessageStream { subscriber }))
    }
}

/// NATS message stream wrapper
struct NatsMessageStream {
    subscriber: Subscriber,
}

#[async_trait]
impl MessageStream for NatsMessageStream {
    async fn next(&mut self) -> Option<ReceivedMessage> {
        use futures::StreamExt;

        self.subscriber.next().await.map(|msg| ReceivedMessage {
            subject: msg.subject.to_string(),
            payload: msg.payload.to_vec(),
            reply: msg.reply.map(|s| s.to_string()),
        })
    }
}
