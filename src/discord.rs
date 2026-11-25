use anyhow::Result;
use serde::Serialize;

#[derive(Serialize)]
struct WebhookMessage<'a> {
    content: &'a str,
}

pub struct DiscordWebhook {
    http: reqwest::Client,
    url: String,
}

impl DiscordWebhook {
    pub fn new(http: reqwest::Client, url: String) -> Self {
        Self { http, url }
    }

    pub async fn send(&self, content: &str) -> Result<()> {
        self.http
            .post(&self.url)
            .json(&WebhookMessage { content })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
