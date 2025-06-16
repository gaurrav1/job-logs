use anyhow::Result;
use crate::config::Config;
use crate::model::{AppState, JobInfo, NotificationBatch};
use crate::services::{
    amazon_service::AmazonService, 
    notification_service::NotificationService,
    persistence_service::PersistenceService,
    telegram_service::TelegramService,
    shutdown_service::ShutdownHandle,
};
use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{self, Duration, MissedTickBehavior};

pub async fn run_job_monitor(
    config: Config,
    shutdown_handle: ShutdownHandle,
) -> Result<()> {
    // Initialize services
    let amazon_service = AmazonService::new(config.clone());
    let telegram_service = TelegramService::new(config.clone());
    let notification_service = NotificationService::new();
    let notification_sender = notification_service.sender();
    
    // Start notification worker
    tokio::spawn({
        let telegram_service = telegram_service.clone();
        async move {
            notification_service.run(telegram_service).await;
        }
    });

    // Load state
    let initial_jobs = PersistenceService::load_seen_jobs(&config.persistence.seen_jobs_file)?;
    info!("Loaded {} seen jobs", initial_jobs.len());
    
    let state = Arc::new(AppState::new(initial_jobs));
    
    // Start persistence service
    tokio::spawn({
        let state = state.clone();
        let config = config.clone();
        async move {
            PersistenceService::run(state, config).await;
        }
    });

    // Start processing loop
    let amazon_service = Arc::new(amazon_service);
    let mut interval = time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    while !shutdown_handle.is_shutdown() {
        interval.tick().await;

        // Process requests with rate limiting
        let mut tasks = Vec::with_capacity(config.rate_limiting.requests_per_second);
        for i in 0..config.rate_limiting.requests_per_second {
            let delay = Duration::from_millis(i as u64 * config.rate_limiting.delay_between_requests_ms);
            let amazon_service = amazon_service.clone();
            let state = state.clone();
            let notification_sender = notification_sender.clone();

            tasks.push(tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                if let Err(e) = process_request(&amazon_service, &state, &notification_sender).await {
                    warn!("Request processing failed: {}", e);
                }
            }));
        }

        for task in tasks {
            let _ = task.await;
        }
    }

    info!("Shutting down job monitor");
    Ok(())
}

async fn process_request(
    amazon_service: &Arc<AmazonService>,
    state: &Arc<AppState>,
    notification_sender: &async_channel::Sender<NotificationBatch>,
) -> Result<()> {
    let jobs = amazon_service.fetch_jobs(state).await?;
    if jobs.is_empty() {
        return Ok(());
    }

    // Group new jobs by location
    let mut new_jobs_by_location: HashMap<String, Vec<JobInfo>> = HashMap::new();
    let mut new_jobs_count = 0;

    for job in jobs {
        if state.add_seen_job(job.id.clone()).await {
            new_jobs_by_location
                .entry(job.location.clone())
                .or_default()
                .push(job.clone());
                
            new_jobs_count += 1;
            
            // Log to console immediately
            log::info!(
                "- {} @ {} @ {} (${:.2}-${:.2}/hr)",
                job.title,
                job.location,
                job.job_type,
                job.pay_min,
                job.pay_max
            );
        }
    }

    if new_jobs_count > 0 {
        log::info!("Found {} new jobs", new_jobs_count);
        
        // Send notifications in batches per location
        for (location, jobs) in new_jobs_by_location {
            let batch = NotificationBatch { location, jobs };
            if let Err(e) = notification_sender.send(batch).await {
                log::error!("Failed to send notification batch: {}", e);
            }
        }
    }

    Ok(())
}