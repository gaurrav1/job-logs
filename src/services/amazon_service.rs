use anyhow::Result;
use crate::config::Config;
use crate::model::{AppState, JobInfo, ApiResponse};
use crate::utils::backoff_strategy;
use chrono::Utc;
use log::{warn};
use rand::random;
use reqwest::Client;
use serde_json::json;

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.107 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Mobile/15E148 Safari/604.1",
];

pub struct AmazonService {
    client: Client,
    config: Config,
}

impl AmazonService {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        AmazonService { client, config }
    }

    pub async fn fetch_jobs(
        &self,
        state: &AppState,
    ) -> Result<Vec<JobInfo>> {
        for attempt in 0..self.config.rate_limiting.max_retries {
            if state.shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
                return Ok(Vec::new());
            }

            match self.try_fetch_jobs().await {
                Ok(jobs) => return Ok(jobs),
                Err(e) => {
                    let delay = backoff_strategy(
                        attempt as u32,
                        self.config.rate_limiting.retry_base_ms,
                        self.config.rate_limiting.retry_max_delay_ms,
                    );
                    
                    warn!(
                        "Attempt {}/{} failed: {}. Retrying in {:?}",
                        attempt + 1,
                        self.config.rate_limiting.max_retries,
                        e,
                        delay
                    );
                    
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        Err(anyhow::anyhow!("All retry attempts exhausted"))
    }

    async fn try_fetch_jobs(&self) -> Result<Vec<JobInfo>> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let user_agent = USER_AGENTS[random::<usize>() % USER_AGENTS.len()];

        let payload = json!({
            "operationName": "searchJobCardsByLocation",
            "variables": {
                "searchJobRequest": {
                    "locale": &self.config.amazon.locale,
                    "country": &self.config.amazon.country,
                    "keyWords": "",
                    "equalFilters": [],
                    "dateFilters": [
                        {
                            "key": "firstDayOnSite",
                            "range": { "startDate": today }
                        }
                    ],
                    "sorters": [
                        { "fieldName": "totalPayRateMax", "ascending": "false" }
                    ],
                    "pageSize": self.config.amazon.page_size
                }
            },
            "query": "query searchJobCardsByLocation($searchJobRequest: SearchJobRequest!) {\n  searchJobCardsByLocation(searchJobRequest: $searchJobRequest) {\n    nextToken\n    jobCards {\n      jobId\n      jobTitle\n      jobType\n      locationName\n    scheduleCount\n      totalPayRateMin\n      totalPayRateMax\n    }\n  }\n}"
        });

        let response = self.client
            .post(&self.config.amazon.api_url)
            .header("User-Agent", user_agent)
            .header("Authorization", format!("Bearer {}", self.config.amazon.api_token))
            .header("Country", &self.config.amazon.country)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!("HTTP error {}: {}", status, body));
        }

        let response_json: ApiResponse = response.json().await?;
        let jobs = response_json.data.search_job_cards.job_cards
            .into_iter()
            .map(|card| JobInfo {
                id: card.id,
                title: card.title,
                location: card.location,
                job_type: card.job_type,
                pay_min: card.pay_min,
                pay_max: card.pay_max,
                shift: card.shift,
            })
            .collect();

        Ok(jobs)
    }
}