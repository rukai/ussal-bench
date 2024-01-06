use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobRequest {
    pub job_id: Uuid,
    pub binary: Vec<u8>,
    pub ty: JobRequestType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum JobRequestType {
    RunBenchCriterion {
        bench_name: String,
    },
    ListBenchesCriterion,
    /// Divan benches run instantly so no point in splitting results across the cluster like we do with criterion
    RunBenchesDivan,
}

/// One JobResponse will be sent per JobRequest
#[derive(Serialize, Deserialize, Debug)]
pub struct JobResponse {
    pub job_id: Uuid,
    pub ty: JobResponseType,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JobResponseType {
    Handshake { machine_type: String },
    RunBench(BenchComplete),
    ListBenches { bench_names: Vec<String> },
    Error(String),
}

impl JobResponseType {
    pub fn get_run_bench(&self) -> Result<&BenchComplete, String> {
        match self {
            JobResponseType::RunBench(x) => Ok(x),
            JobResponseType::ListBenches { .. } => {
                Err("Unexpected response ListBenches".to_owned())
            }
            JobResponseType::Handshake { .. } => Err("Unexpected handshake".to_owned()),
            JobResponseType::Error(err) => Err(err.clone()),
        }
    }

    pub fn get_list_benches(&self) -> Result<&Vec<String>, String> {
        match self {
            JobResponseType::ListBenches { bench_names, .. } => Ok(bench_names),
            JobResponseType::RunBench(_) => Err("Unexpected response RunBench".to_owned()),
            JobResponseType::Handshake { .. } => Err("Unexpected handshake".to_owned()),
            JobResponseType::Error(err) => Err(err.clone()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchComplete {
    pub wall_time: f32,
}
