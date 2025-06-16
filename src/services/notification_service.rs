use async_channel::{bounded, Receiver, Sender};
use crate::model::{NotificationBatch};
use crate::services::telegram_service::TelegramService;
use log::info;

pub struct NotificationService {
    sender: Sender<NotificationBatch>,
    receiver: Receiver<NotificationBatch>,
}

impl NotificationService {
    pub fn new() -> Self {
        let (sender, receiver) = bounded(100);
        NotificationService { sender, receiver }
    }

    pub fn sender(&self) -> Sender<NotificationBatch> {
        self.sender.clone()
    }

    pub async fn run(&self, telegram_service: TelegramService) {
        while let Ok(batch) = self.receiver.recv().await {
            if let Err(e) = telegram_service.send_batch(&batch).await {
                log::error!("Failed to send notification: {}", e);
            } else {
                info!("Sent notification for {} jobs in {}", batch.jobs.len(), batch.location);
            }
        }
    }
}