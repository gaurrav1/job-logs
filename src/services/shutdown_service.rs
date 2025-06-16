use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::signal;
use log::info;

#[derive(Clone)]
pub struct ShutdownHandle {
    flag: Arc<AtomicBool>,
}

impl ShutdownHandle {
    pub fn is_shutdown(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}

pub struct ShutdownService {
    handle: ShutdownHandle,
}

impl ShutdownService {
    pub fn new() -> Self {
        ShutdownService {
            handle: ShutdownHandle {
                flag: Arc::new(AtomicBool::new(false)),
            },
        }
    }

    pub fn handle(&self) -> ShutdownHandle {
        self.handle.clone()
    }

    pub async fn wait_for_shutdown(&self) -> anyhow::Result<()> {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Shutdown signal received");
                self.handle.flag.store(true, Ordering::Relaxed);
                Ok(())
            }
            Err(err) => Err(anyhow::anyhow!("Unable to listen for shutdown signal: {}", err)),
        }
    }
}