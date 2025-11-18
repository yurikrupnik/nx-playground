//! Email worker that processes emails from Redis Streams

use crate::config::EmailConfig;
use crate::state::AppState;
use email::models::{Email, EmailEvent};
use email::provider::EmailProvider;
use email::stream::EmailConsumer;
use eyre::Result;

/// Run the email worker
pub async fn run_worker(state: AppState, config: EmailConfig, worker_id: usize) -> Result<()> {
    let consumer_name = format!("worker-{}", worker_id);
    let consumer = EmailConsumer::new(state.redis.clone(), &consumer_name);

    // Initialize consumer group (idempotent)
    consumer.init_consumer_group().await?;

    tracing::info!(
        worker_id = worker_id,
        consumer_name = %consumer_name,
        "Email worker started"
    );

    // On startup, process any pending messages assigned to this consumer
    if let Ok(pending_ids) = consumer.get_pending(config.batch_size).await {
        if !pending_ids.is_empty() {
            tracing::info!(
                count = pending_ids.len(),
                "Found pending messages to process"
            );
        }
    }

    loop {
        // Read emails from stream
        match consumer
            .read_emails(config.batch_size, config.block_timeout_ms)
            .await
        {
            Ok(emails) => {
                if !emails.is_empty() {
                    tracing::info!(count = emails.len(), "Received emails from stream");
                }
                for (stream_id, event) in emails {
                    tracing::debug!(stream_id = %stream_id, "Processing event");
                    match event {
                        EmailEvent::SendEmail(email) => {
                            process_email(&state, &consumer, &stream_id, email).await;
                        }
                        _ => {
                            // Other events (EmailSent, EmailFailed) are for monitoring
                            // Just acknowledge them
                            if let Err(e) = consumer.ack(&stream_id).await {
                                tracing::error!(error = %e, "Failed to ack event");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = ?e, "Failed to read from stream");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }

        // Periodically claim old messages from dead workers
        if worker_id == 0 {
            // Only first worker does this
            if let Err(e) = claim_and_process_old_messages(&state, &consumer, &config).await {
                tracing::error!(error = ?e, "Failed to claim old messages");
            }
        }
    }
}

async fn process_email(state: &AppState, consumer: &EmailConsumer, stream_id: &str, email: Email) {
    tracing::info!(
        email_id = %email.id,
        to = %email.to,
        subject = %email.subject,
        "Processing email"
    );

    // Render template if specified
    let mut email = if let Some(template_name) = &email.template {
        match render_template(state, &email, template_name).await {
            Ok(rendered) => rendered,
            Err(e) => {
                tracing::error!(
                    email_id = %email.id,
                    template = %template_name,
                    error = %e,
                    "Failed to render template"
                );
                // Ack the message since this is a permanent error
                let _ = consumer.ack(stream_id).await;
                return;
            }
        }
    } else {
        email
    };

    // Send the email
    match state.provider.send(&email).await {
        Ok(result) => {
            tracing::info!(
                email_id = %email.id,
                message_id = %result.message_id,
                "Email sent successfully"
            );

            // Acknowledge the message
            if let Err(e) = consumer.ack(stream_id).await {
                tracing::error!(
                    stream_id = %stream_id,
                    error = %e,
                    "Failed to acknowledge message"
                );
            }
        }
        Err(e) => {
            tracing::error!(
                email_id = %email.id,
                error = %e,
                retry_count = email.retry_count,
                "Failed to send email"
            );

            if email.can_retry() {
                // Re-queue for retry
                email.increment_retry();
                let producer = email::stream::EmailProducer::new(state.redis.clone());
                if let Err(e) = producer.send(email.clone()).await {
                    tracing::error!(error = %e, "Failed to re-queue email for retry");
                }
            }

            // Acknowledge the original message (we've re-queued if needed)
            let _ = consumer.ack(stream_id).await;
        }
    }
}

async fn render_template(state: &AppState, email: &Email, template_name: &str) -> Result<Email> {
    let template = state
        .templates
        .get(template_name)
        .await?
        .ok_or_else(|| eyre::eyre!("Template not found: {}", template_name))?;

    let rendered = template.render(&email.template_data)?;

    let mut email = email.clone();
    email.subject = rendered.subject;
    email.body_text = rendered.body_text;
    email.body_html = rendered.body_html;
    email.template = None; // Clear template so we don't re-render

    Ok(email)
}

async fn claim_and_process_old_messages(
    state: &AppState,
    consumer: &EmailConsumer,
    config: &EmailConfig,
) -> Result<()> {
    // Claim messages idle for more than 5 minutes
    let old_messages = consumer
        .claim_old_messages(5 * 60 * 1000, config.batch_size)
        .await?;

    for (stream_id, event) in old_messages {
        if let EmailEvent::SendEmail(email) = event {
            tracing::info!(
                email_id = %email.id,
                stream_id = %stream_id,
                "Processing claimed message"
            );
            process_email(state, consumer, &stream_id, email).await;
        }
    }

    Ok(())
}
