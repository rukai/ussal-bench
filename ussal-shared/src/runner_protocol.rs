use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct JobRequest {
    pub job_id: Uuid,
    pub binary: Vec<u8>,
    pub ty: JobRequestType,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JobRequestType {
    RunBench { bench_name: String },
    ListBenches,
}

/// One JobResponse will be sent per JobRequest
#[derive(Serialize, Deserialize, Debug)]
pub struct JobResponse {
    pub job_id: Uuid,
    pub ty: JobResponseType,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JobResponseType {
    RunBench(BenchComplete),
    ListBenches(Vec<String>),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchComplete {
    pub wall_time: f32,
}
