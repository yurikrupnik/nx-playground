//! Redis Streams integration for email processing

use crate::models::{Email, EmailEvent};
use eyre::{Result, WrapErr};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult};
use serde_json;

/// Stream names
pub const EMAIL_STREAM: &str = "notifications:email:stream";
pub const EMAIL_CONSUMER_GROUP: &str = "email-workers";

/// Producer for adding emails to the stream
#[derive(Clone)]
pub struct EmailProducer {
    redis: ConnectionManager,
}

impl EmailProducer {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    /// Add an email to the stream for processing
    pub async fn send(&self, email: Email) -> Result<String> {
        let mut conn = self.redis.clone();
        let event = EmailEvent::SendEmail(email.clone());
        let payload = serde_json::to_string(&event)?;

        let message_id: String = redis::cmd("XADD")
            .arg(EMAIL_STREAM)
            .arg("*")
            .arg("event")
            .arg(&payload)
            .arg("priority")
            .arg(serde_json::to_string(&email.priority)?)
            .arg("email_id")
            .arg(&email.id)
            .query_async(&mut conn)
            .await
            .wrap_err("Failed to add email to stream")?;

        tracing::info!(
            email_id = %email.id,
            stream_id = %message_id,
            "Email added to stream"
        );

        Ok(message_id)
    }

    /// Publish an email event (sent, failed, etc.)
    pub async fn publish_event(&self, event: EmailEvent) -> Result<String> {
        let mut conn = self.redis.clone();
        let payload = serde_json::to_string(&event)?;

        let message_id: String = redis::cmd("XADD")
            .arg(EMAIL_STREAM)
            .arg("*")
            .arg("event")
            .arg(&payload)
            .query_async(&mut conn)
            .await
            .wrap_err("Failed to publish email event")?;

        Ok(message_id)
    }
}

/// Consumer for processing emails from the stream
pub struct EmailConsumer {
    redis: ConnectionManager,
    consumer_name: String,
}

impl EmailConsumer {
    pub fn new(redis: ConnectionManager, consumer_name: impl Into<String>) -> Self {
        Self {
            redis,
            consumer_name: consumer_name.into(),
        }
    }

    /// Initialize the consumer group (call once at startup)
    pub async fn init_consumer_group(&self) -> Result<()> {
        let mut conn = self.redis.clone();

        tracing::info!(
            stream = EMAIL_STREAM,
            group = EMAIL_CONSUMER_GROUP,
            consumer = %self.consumer_name,
            "Initializing consumer group"
        );

        // Create consumer group, ignore error if it already exists
        // Retry on timeout
        let mut retries = 3;
        loop {
            let result: RedisResult<String> = redis::cmd("XGROUP")
                .arg("CREATE")
                .arg(EMAIL_STREAM)
                .arg(EMAIL_CONSUMER_GROUP)
                .arg("0")
                .arg("MKSTREAM")
                .query_async(&mut conn)
                .await;

            match result {
                Ok(_) => {
                    tracing::info!(
                        stream = EMAIL_STREAM,
                        group = EMAIL_CONSUMER_GROUP,
                        "Created consumer group"
                    );
                    break;
                }
                Err(e) if e.to_string().contains("BUSYGROUP") => {
                    tracing::info!(
                        stream = EMAIL_STREAM,
                        group = EMAIL_CONSUMER_GROUP,
                        "Consumer group already exists"
                    );
                    break;
                }
                Err(e) if e.is_timeout() && retries > 0 => {
                    tracing::warn!(
                        retries_left = retries,
                        "Timeout creating consumer group, retrying..."
                    );
                    retries -= 1;
                    continue;
                }
                Err(e) => {
                    tracing::error!(
                        stream = EMAIL_STREAM,
                        group = EMAIL_CONSUMER_GROUP,
                        error = %e,
                        "Failed to create consumer group"
                    );
                    return Err(e).wrap_err("Failed to create consumer group");
                }
            }
        }

        Ok(())
    }

    /// Read emails from the stream
    /// Returns a list of (stream_id, EmailEvent) tuples
    pub async fn read_emails(
        &self,
        count: usize,
        block_ms: usize,
    ) -> Result<Vec<(String, EmailEvent)>> {
        let mut conn = self.redis.clone();

        // XREADGROUP GROUP email-workers consumer-name COUNT 10 BLOCK 5000 STREAMS emails:stream >
        // Use Option because BLOCK returns nil when timeout expires with no messages
        let results: Option<redis::streams::StreamReadReply> = match redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(EMAIL_CONSUMER_GROUP)
            .arg(&self.consumer_name)
            .arg("COUNT")
            .arg(count)
            .arg("BLOCK")
            .arg(block_ms)
            .arg("STREAMS")
            .arg(EMAIL_STREAM)
            .arg(">")
            .query_async(&mut conn)
            .await
        {
            Ok(result) => {
                tracing::debug!("XREADGROUP returned result");
                result
            }
            Err(e) => {
                // Timeout errors are expected when using BLOCK - just return empty
                if e.is_timeout() {
                    tracing::debug!("XREADGROUP timeout - no new messages");
                    return Ok(Vec::new());
                }
                return Err(e).wrap_err("Failed to read from stream");
            }
        };

        let mut emails = Vec::new();

        // Handle nil response (timeout with no messages)
        let Some(results) = results else {
            return Ok(emails);
        };

        for stream_key in results.keys {
            tracing::debug!(stream = %stream_key.key, count = stream_key.ids.len(), "Processing stream key");
            for stream_id in stream_key.ids {
                tracing::debug!(stream_id = %stream_id.id, fields = ?stream_id.map.keys().collect::<Vec<_>>(), "Processing message");
                if let Some(event_data) = stream_id.map.get("event") {
                    if let redis::Value::BulkString(bytes) = event_data {
                        let event_str = String::from_utf8_lossy(bytes);
                        tracing::debug!(event_str = %event_str, "Deserializing event");
                        match serde_json::from_str::<EmailEvent>(&event_str) {
                            Ok(event) => {
                                tracing::debug!(stream_id = %stream_id.id, "Event deserialized successfully");
                                emails.push((stream_id.id.clone(), event));
                            }
                            Err(e) => {
                                tracing::error!(
                                    stream_id = %stream_id.id,
                                    error = %e,
                                    event_str = %event_str,
                                    "Failed to deserialize email event"
                                );
                            }
                        }
                    } else {
                        tracing::warn!(stream_id = %stream_id.id, value_type = ?event_data, "Event field is not a BulkString");
                    }
                } else {
                    tracing::warn!(stream_id = %stream_id.id, "Message has no 'event' field");
                }
            }
        }

        Ok(emails)
    }

    /// Acknowledge that an email has been processed
    pub async fn ack(&self, stream_id: &str) -> Result<()> {
        let mut conn = self.redis.clone();

        let _: i64 = conn
            .xack(EMAIL_STREAM, EMAIL_CONSUMER_GROUP, &[stream_id])
            .await
            .wrap_err("Failed to acknowledge message")?;

        tracing::debug!(stream_id = %stream_id, "Message acknowledged");
        Ok(())
    }

    /// Get pending messages (for recovery/retry)
    pub async fn get_pending(&self, count: usize) -> Result<Vec<String>> {
        let mut conn = self.redis.clone();

        // XPENDING email:stream email-workers - + count consumer-name
        let pending: redis::streams::StreamPendingCountReply = redis::cmd("XPENDING")
            .arg(EMAIL_STREAM)
            .arg(EMAIL_CONSUMER_GROUP)
            .arg("-")
            .arg("+")
            .arg(count)
            .arg(&self.consumer_name)
            .query_async(&mut conn)
            .await
            .wrap_err("Failed to get pending messages")?;

        Ok(pending.ids.into_iter().map(|p| p.id).collect())
    }

    /// Claim old pending messages from other consumers
    pub async fn claim_old_messages(
        &self,
        min_idle_ms: usize,
        count: usize,
    ) -> Result<Vec<(String, EmailEvent)>> {
        let mut conn = self.redis.clone();

        // XAUTOCLAIM email:stream email-workers consumer-name min-idle-time 0 COUNT count
        let results: Option<redis::streams::StreamAutoClaimReply> = match redis::cmd("XAUTOCLAIM")
            .arg(EMAIL_STREAM)
            .arg(EMAIL_CONSUMER_GROUP)
            .arg(&self.consumer_name)
            .arg(min_idle_ms)
            .arg("0")
            .arg("COUNT")
            .arg(count)
            .query_async(&mut conn)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                // Timeout errors can happen - just return empty
                if e.is_timeout() {
                    return Ok(Vec::new());
                }
                return Err(e).wrap_err("Failed to autoclaim messages");
            }
        };

        let mut emails = Vec::new();

        let Some(results) = results else {
            return Ok(emails);
        };

        for stream_id in results.claimed {
            if let Some(event_data) = stream_id.map.get("event") {
                if let redis::Value::BulkString(bytes) = event_data {
                    let event_str = String::from_utf8_lossy(bytes);
                    if let Ok(event) = serde_json::from_str::<EmailEvent>(&event_str) {
                        emails.push((stream_id.id.clone(), event));
                    }
                }
            }
        }

        Ok(emails)
    }
}

/// Utility to get stream info
pub async fn get_stream_info(redis: &mut ConnectionManager) -> Result<StreamInfo> {
    let info: redis::streams::StreamInfoStreamReply = redis::cmd("XINFO")
        .arg("STREAM")
        .arg(EMAIL_STREAM)
        .query_async(redis)
        .await
        .wrap_err("Failed to get stream info")?;

    Ok(StreamInfo {
        length: info.length,
        first_entry_id: if info.first_entry.id.is_empty() {
            None
        } else {
            Some(info.first_entry.id)
        },
        last_entry_id: if info.last_entry.id.is_empty() {
            None
        } else {
            Some(info.last_entry.id)
        },
    })
}

#[derive(Debug)]
pub struct StreamInfo {
    pub length: usize,
    pub first_entry_id: Option<String>,
    pub last_entry_id: Option<String>,
}
