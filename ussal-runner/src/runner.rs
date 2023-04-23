use crate::system::run_command;
use ussal_shared::runner_protocol::{BenchComplete, JobRequest, JobResponse, JobResult};

pub fn run_job_request(request: &JobRequest) -> JobResponse {
    // TODO: write binary to tmpfs and run as ussal-sandbox
    std::fs::write("/home/ussal-runner/binary-under-test", &request.binary).unwrap();
    run_command("chmod", &["+x", "/home/ussal-runner/binary-under-test"]).unwrap();

    let output = run_command("/home/ussal-runner/binary-under-test", &[]).unwrap();
    tracing::info!("request {}", output);

    JobResponse {
        job_id: request.job_id,
        result: JobResult::BenchComplete(BenchComplete {
            //bench_name: "CoolBench".to_owned(),
            bench_name: output,
            wall_time: 413.,
        }),
    }
}
