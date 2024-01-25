use crate::{cli::SandboxMode, system::run_sandboxed_binary};
use anyhow::{anyhow, Result};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use ussal_networking::runner_protocol::{
    BenchComplete, JobRequest, JobRequestType, JobResponse, JobResponseType,
};
use uuid::Uuid;

pub async fn runner(sandbox_mode: SandboxMode, address: &str, machine_type: &str) {
    loop {
        let stream = match connect(address).await {
            Ok(stream) => stream,
            Err(error) => {
                tracing::error!(
                    "{:?}",
                    error.context("Failed to connect to orchestrator, retrying in 60s")
                );
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        };
        let (tx, mut rx) =
            ussal_networking::spawn_read_write_tasks::<JobResponse, JobRequest>(stream).await;
        tx.send(JobResponse {
            job_id: Uuid::new_v4(),
            ty: JobResponseType::Handshake {
                machine_type: machine_type.to_owned(),
            },
        })
        .unwrap();
        if let Some(request) = rx.recv().await {
            tracing::info!("running job: {} {:?}", request.job_id, request.ty);
            let response = run_job_request(sandbox_mode, &request);
            tx.send(response).unwrap();
        } else {
            tracing::error!("Connection was killed, retrying in 60s");
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}

async fn connect(uri: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let (ws_stream, _) = timeout(Duration::from_secs(10), connect_async(uri))
        .await
        .map_err(|_| anyhow!("Timed out connecting to {uri} after 10 seconds"))?
        .map_err(|e| anyhow!(e).context(format!("Failed to connect to {uri}")))?;
    Ok(ws_stream)
}

pub fn run_job_request(sandbox_mode: SandboxMode, request: &JobRequest) -> JobResponse {
    // TODO: run as ussal-sandbox
    let binary_path = std::env::temp_dir().join("binary-under-test");
    std::fs::remove_file(&binary_path).ok();
    std::fs::write(&binary_path, &request.binary).unwrap();
    #[cfg(unix)]
    std::fs::set_permissions(
        &binary_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o100),
    )
    .unwrap();

    let binary_path = binary_path.to_str().unwrap();

    match &request.ty {
        JobRequestType::ListBenchesCriterion => {
            // `cargo bench` automatically adds in the `--bench`
            let output =
                run_sandboxed_binary(sandbox_mode, binary_path, &["--bench", "--list"]).unwrap();

            let bench_names: Vec<String> = output
                .lines()
                .filter_map(|line| line.strip_suffix(": benchmark").map(|x| x.to_owned()))
                .collect();
            JobResponse {
                job_id: request.job_id,
                ty: JobResponseType::ListBenches { bench_names },
            }
        }
        JobRequestType::RunBenchCriterion { bench_name } => {
            let output = run_sandboxed_binary(
                sandbox_mode,
                binary_path,
                &[
                    "--bench",
                    "--exact",
                    bench_name,
                    "--noplot",
                    "--discard-baseline",
                ],
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
        JobRequestType::RunBenchesDivan => {
            todo!("implement divan support")
        }
    }
}
