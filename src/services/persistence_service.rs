use anyhow::Result;
use crate::model::AppState;
use crate::config::Config;
use log::{info, warn};
use std::collections::HashSet;
use std::fs;
use std::time::Duration;
use tokio::time;
use std::sync::Arc;

pub struct PersistenceService;

impl PersistenceService {
    pub async fn run(state: Arc<AppState>, config: Config) {
        let mut interval = time::interval(Duration::from_secs(config.persistence.persist_interval_secs));
        
        while !state.shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
            interval.tick().await;
            
            let jobs = state.get_seen_jobs().await;
            match Self::save_seen_jobs(&config.persistence.seen_jobs_file, &jobs) {
                Ok(_) => info!("Persisted {} seen jobs to disk", jobs.len()),
                Err(e) => warn!("Failed to persist jobs: {}", e),
            }
        }
        
        // Final persistence on shutdown
        let jobs = state.get_seen_jobs().await;
        if let Err(e) = Self::save_seen_jobs(&config.persistence.seen_jobs_file, &jobs) {
            warn!("Final persistence failed: {}", e);
        }
    }

    pub fn load_seen_jobs(path: &str) -> Result<HashSet<String>> {
        fs::read_to_string(path)
            .map(|contents| {
                contents
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .or_else(|_| {
                warn!("No seen jobs file found, starting fresh");
                Ok(HashSet::new())
            })
    }

    fn save_seen_jobs(path: &str, seen_jobs: &HashSet<String>) -> Result<()> {
        let data: Vec<&str> = seen_jobs.iter().map(|s| s.as_str()).collect();
        fs::write(path, data.join("\n"))?;
        Ok(())
    }
}