use anyhow::Result;
use crate::config::Config;
use crate::model::{NotificationBatch};
use crate::utils::{escape_html, humanize_job_type};
use reqwest::Client;

#[derive(Clone)]
pub struct TelegramService {
    client: Client,
    config: Config,
}

impl TelegramService {
    pub fn new(config: Config) -> Self {
        TelegramService {
            client: Client::new(),
            config,
        }
    }

    pub async fn send_batch(&self, batch: &NotificationBatch) -> Result<()> {
        let mut message = format!(
            "<b>New Jobs in {}</b>\n═══════════════════\n",
            escape_html(&batch.location)
        );

        for job in &batch.jobs {
            let job_type = humanize_job_type(&job.job_type);
            message.push_str(&format!(
                "<b>{}</b>\n- Type: {}\n- Shifts: {}\n- Pay: ${:.2}-${:.2}/hr\n═══════════════════\n",
                escape_html(&job.title),
                job_type,
                job.shift,
                job.pay_min,
                job.pay_max
            ));
        }

        self.send_alert(&message).await
    }

    async fn send_alert(&self, message: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.telegram.bot_token
        );

        let payload = serde_json::json!({
            "chat_id": self.config.telegram.chat_id,
            "text": message,
            "parse_mode": "HTML",
            "disable_web_page_preview": true
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(anyhow::anyhow!("Telegram API error: {}", text));
        }

        Ok(())
    }
}