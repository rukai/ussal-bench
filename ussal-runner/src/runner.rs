use crate::system::run_command;
use ussal_shared::runner_protocol::{BenchComplete, JobRequest, JobResponse};

pub fn run_job_request(request: &JobRequest) -> JobResponse {
    // TODO: write binary to tmpfs and run as ussal-sandbox
    std::fs::write("/home/ussal-runner/binary-under-test", &request.binary).unwrap();
    run_command("chmod", &["+x", "/home/ussal-runner/binary-under-test"]).unwrap();

    let output = run_command(
        "/home/ussal-runner/binary-under-test",
        &["--bench", &request.bench_name],
    )
    .unwrap();
    tracing::info!("request {}", output);

    let mut wall_time: Option<f32> = None;
    // This logic is so brittle, but we plan to replace criterion later anyway.
    for line in output.lines() {
        if line.contains("time: ") {
            let mut iter = line.split('[');
            iter.next().unwrap(); // skip first one

            let mut words = iter.next().unwrap().split_whitespace();
            words.next().unwrap(); // skip these
            words.next().unwrap(); // skip these
            let value = words.next().unwrap();
            wall_time = Some(
                value
                    .parse()
                    .unwrap_or_else(|_| panic!("Failed to parse {value} as float")),
            );
        }
    }

    let wall_time = wall_time.unwrap_or(0.0); // TODO: this should really panic on missing
    JobResponse {
        job_id: request.job_id,
        result: Ok(BenchComplete { wall_time }),
    }
}
