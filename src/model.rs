use serde::Deserialize;
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct JobInfo {
    pub id: String,
    pub title: String,
    pub location: String,
    pub job_type: String,
    pub pay_min: f64,
    pub pay_max: f64,
    pub shift: i64,
}

#[derive(Deserialize)]
pub struct ApiResponse {
    pub data: ApiData,
}

#[derive(Deserialize)]
pub struct ApiData {
    #[serde(rename = "searchJobCardsByLocation")]
    pub search_job_cards: SearchJobCards,
}

#[derive(Deserialize)]
pub struct SearchJobCards {
    #[serde(rename = "jobCards")]
    pub job_cards: Vec<JobCard>,
}

#[derive(Deserialize)]
pub struct JobCard {
    #[serde(rename = "jobId")]
    pub id: String,
    #[serde(rename = "jobTitle")]
    pub title: String,
    #[serde(rename = "jobType")]
    pub job_type: String,
    #[serde(rename = "locationName")]
    pub location: String,
    #[serde(rename = "scheduleCount")]
    pub shift: i64,
    #[serde(rename = "totalPayRateMin")]
    pub pay_min: f64,
    #[serde(rename = "totalPayRateMax")]
    pub pay_max: f64,
}

pub struct AppState {
    pub seen_jobs: Mutex<HashSet<String>>,
    pub shutdown_flag: AtomicBool,
}

impl AppState {
    pub fn new(initial_jobs: HashSet<String>) -> Self {
        AppState {
            seen_jobs: Mutex::new(initial_jobs),
            shutdown_flag: AtomicBool::new(false),
        }
    }

    pub async fn add_seen_job(&self, job_id: String) -> bool {
        let mut jobs = self.seen_jobs.lock().await;
        jobs.insert(job_id)
    }

    pub async fn get_seen_jobs(&self) -> HashSet<String> {
        self.seen_jobs.lock().await.clone()
    }
}

pub struct NotificationBatch {
    pub location: String,
    pub jobs: Vec<JobInfo>,
}