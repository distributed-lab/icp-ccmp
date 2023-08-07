use std::time::Duration;

use ic_cdk_timers::{TimerId, set_timer_interval, clear_timer};

use serde::{Deserialize, Serialize};
use candid::CandidType;

use crate::jobs::{listener, signer, writer};

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub enum JobType {
    Listener,
    Signer,
    Writer,
    #[default]
    Unknown,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct Job {
    pub interval_secs: u64,
    timer_id: String,
    is_active: bool,
    job_type: JobType,
}

impl Job {
    pub fn new(interval_secs: u64, job_type: JobType) -> Self {
        Self {
            interval_secs,
            timer_id: String::new(),
            is_active: false,
            job_type,
        }
    }

    pub fn run(&mut self) {
        if self.is_active {
            return;
        }

        let func = match self.job_type {
            JobType::Listener => listener::run,
            JobType::Signer => signer::run,
            JobType::Writer => writer::run,
            _ => panic!("Unknown job type"),
        };

        let timer_id = set_timer_interval(Duration::from_secs(self.interval_secs), func);
        let serialized_timer_id = serde_json::to_string(&timer_id).unwrap();

        self.timer_id = serialized_timer_id;

        self.is_active = true;
    }

    pub fn stop(&mut self) {
        if !self.is_active {
            return;
        }

        let timer_id: TimerId = serde_json::from_str(&self.timer_id).unwrap();

        clear_timer(timer_id);

        self.is_active = false;
    }

    pub fn update_interval_secs(&mut self, interval_secs: u64) {
        self.stop();

        self.interval_secs = interval_secs;

        self.run();
    }
}