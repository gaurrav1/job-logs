mod config;
mod model;
mod services;
mod controllers;
mod logging;
mod utils;

use anyhow::Result;
use config::Config;
use controllers::job_monitor_controller::run_job_monitor;
use services::shutdown_service::ShutdownService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    logging::init_logger();

    // Load configuration
    let config = Config::load()?;

    // Setup shutdown service
    let shutdown_service = ShutdownService::new();
    let shutdown_handle = shutdown_service.handle();

    // Start job monitor
    tokio::spawn(async move {
        if let Err(e) = run_job_monitor(config, shutdown_handle.clone()).await {
            log::error!("Job monitor failed: {}", e);
        }
    });

    // Wait for shutdown signal
    shutdown_service.wait_for_shutdown().await?;

    Ok(())
}