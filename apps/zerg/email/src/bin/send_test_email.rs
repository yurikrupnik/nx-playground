//! Test binary to send a test email via Redis Streams

use config::FromEnv;
use email::{Email, EmailPriority, EmailProducer};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Get Redis host from env
    let redis_host =
        std::env::var("REDIS_HOST").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    println!("Connecting to Redis at {}", redis_host);

    // Connect to Redis
    let redis = services::redis::connect(&redis_host).await?;
    let producer = EmailProducer::new(redis);

    // Create test email
    let email = Email::new("test@example.com", "Test Email from Zerg")
        .with_text("Hello! This is a test email sent via Redis Streams.")
        .with_html("<h1>Hello!</h1><p>This is a test email sent via Redis Streams.</p>")
        .with_priority(EmailPriority::High);

    println!("Sending test email to: {}", email.to);
    println!("Subject: {}", email.subject);

    // Send to stream
    let stream_id = producer.send(email).await?;

    println!("Email queued successfully!");
    println!("Stream ID: {}", stream_id);

    Ok(())
}
