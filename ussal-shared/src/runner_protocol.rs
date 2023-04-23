use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct JobRequest {
    pub job_id: Uuid,
    pub binary: Vec<u8>,
    pub bench_name: String,
}

/// Multiple JobResponses will be sent per JobRequest
#[derive(Serialize, Deserialize, Debug)]
pub struct JobResponse {
    pub job_id: Uuid,
    pub result: Result<BenchComplete, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchComplete {
    pub wall_time: f32,
}
