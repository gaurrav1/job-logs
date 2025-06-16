use anyhow::{Context, Result};
use serde::Deserialize;
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub amazon: AmazonConfig,
    pub telegram: TelegramConfig,
    pub persistence: PersistenceConfig,
    pub rate_limiting: RateLimitingConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AmazonConfig {
    pub api_url: String,
    pub api_token: String,
    pub country: String,
    pub locale: String,
    pub page_size: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PersistenceConfig {
    pub seen_jobs_file: String,
    pub persist_interval_secs: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RateLimitingConfig {
    pub requests_per_second: usize,
    pub delay_between_requests_ms: u64,
    pub retry_base_ms: u64,
    pub retry_max_delay_ms: u64,
    pub max_retries: usize,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = "config.toml";
        if !Path::new(config_path).exists() {
            Self::create_default_config(config_path)?;
        }

        let config_content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path))?;
        
        toml::from_str(&config_content)
            .with_context(|| "Failed to parse config file")
    }

    fn create_default_config(path: &str) -> Result<()> {
        let default_config = Config {
            amazon: AmazonConfig {
                api_url: "https://e5mquma77feepi2.amazonaws.com/graphql".into(),
                api_token: "YOUR_API_TOKEN".into(),
                country: "Canada".into(),
                locale: "en-US".into(),
                page_size: 100,
            },
            telegram: TelegramConfig {
                bot_token: "YOUR_BOT_TOKEN".into(),
                chat_id: "YOUR_CHAT_ID".into(),
            },
            persistence: PersistenceConfig {
                seen_jobs_file: "seen_jobs.txt".into(),
                persist_interval_secs: 300,
            },
            rate_limiting: RateLimitingConfig {
                requests_per_second: 2,
                delay_between_requests_ms: 300,
                retry_base_ms: 500,
                retry_max_delay_ms: 10_000,
                max_retries: 5,
            },
        };

        let toml = toml::to_string_pretty(&default_config)?;
        fs::write(path, toml)?;
        
        log::warn!("Created default config file. Please update with your credentials.");
        Ok(())
    }
}