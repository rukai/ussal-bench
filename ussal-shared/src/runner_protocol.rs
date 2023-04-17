use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct JobRequest {
    pub auth_token: Uuid,
    pub job_id: Uuid,
    pub binary: Vec<u8>,
    pub os: String,
    pub arch: String,
}

/// Multiple JobResponses will be sent per JobRequest
#[derive(Serialize, Deserialize, Debug)]
pub struct JobResponse {
    pub job_id: Uuid,
    pub result: JobResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JobResult {
    /// Single bench completed
    BenchComplete(BenchComplete),
    /// Single bench failed
    BenchError(String),
    /// Entire job succesfully completed
    JobComplete,
    /// Entire job failed
    JobError(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchComplete {
    pub bench_name: String,
    pub wall_time: f32,
}
