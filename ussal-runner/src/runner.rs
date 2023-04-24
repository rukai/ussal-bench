use crate::system::run_command;
use ussal_shared::runner_protocol::{
    BenchComplete, JobRequest, JobRequestType, JobResponse, JobResponseType,
};

pub fn run_job_request(request: &JobRequest) -> JobResponse {
    // TODO: write binary to tmpfs and run as ussal-sandbox
    std::fs::write("/home/ussal-runner/binary-under-test", &request.binary).unwrap();
    run_command("chmod", &["+x", "/home/ussal-runner/binary-under-test"]).unwrap();

    match &request.ty {
        JobRequestType::ListBenches => {
            // `cargo bench` automatically adds in the `--bench`
            let output = run_command(
                "/home/ussal-runner/binary-under-test",
                &["--bench", "--list"],
            )
            .unwrap();

            let benches: Vec<String> = output
                .lines()
                .filter_map(|line| line.strip_suffix(": bench").map(|x| x.to_owned()))
                .collect();
            JobResponse {
                job_id: request.job_id,
                ty: JobResponseType::ListBenches(benches),
            }
        }
        JobRequestType::RunBench { bench_name } => {
            let output = run_command(
                "/home/ussal-runner/binary-under-test",
                &["--bench", bench_name],
            )
            .unwrap();

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

            let wall_time =
                wall_time.unwrap_or_else(|| panic!("Did not find wall time in output: {output:?}"));
            JobResponse {
                job_id: request.job_id,
                ty: JobResponseType::RunBench(BenchComplete { wall_time }),
            }
        }
    }
}
