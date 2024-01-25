use crate::{
    cli::Args,
    config::{BenchFramework, Config},
};
use anyhow::{anyhow, Result};
use cargo_metadata::{Message, MetadataCommand};
use std::process::{Command, Stdio};
use ussal_networking::orchestrator_protocol::{BencherCrate, JobRequest};
use uuid::Uuid;

pub fn get_jobs(args: &Args, config: &Config) -> Result<Vec<JobRequest>> {
    // Run the command to stdout once so the user can see it.
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut jobs = vec![];

    for run in &config.runs {
        println!("Building benches for {:?}", run.target_triple);
        let mut target_dir = MetadataCommand::new().exec().unwrap().target_directory;
        target_dir.push(format!("ussal_target_{}", &run.target_triple));
        let mut command = Command::new(&cargo)
            .args([
                "build",
                "--benches",
                "--profile",
                "bench",
                "--target",
                &run.target_triple,
                "--message-format=json-render-diagnostics",
                // It is common to setup a faster linker such as mold or lld to run for just your native target.
                // It cant be set for some targets as they dont support building with these linkers.
                // This results in a separate rustflags value for native and other targets.
                // Currently rust triggers a full rebuild every time the rustflags value changes.
                //
                // Therefore we have this hack where we use a different target dir for each target-triple to avoid constantly triggering full rebuilds.
                // When this issue is resolved we might be able to remove this hack: https://github.com/rust-lang/cargo/issues/8716
                "--target-dir",
                target_dir.as_ref(),
            ])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let reader = std::io::BufReader::new(command.stdout.take().unwrap());
        for message in cargo_metadata::Message::parse_stream(reader) {
            if let Message::CompilerArtifact(artifact) = message.unwrap() {
                if artifact.target.is_bench() {
                    if let Some(binary) = artifact.executable {
                        let bench_name = binary.as_str().rsplit_once('-').unwrap().0;
                        if let Some(bench) = config.benches.iter().find(|x| x.name == bench_name) {
                            jobs.push(JobRequest {
                                bencher_crate: match bench.framework {
                                    BenchFramework::Criterion => BencherCrate::Criterion,
                                    BenchFramework::Divan => BencherCrate::Divan,
                                },
                                auth_token: args.auth_token,
                                job_id: Uuid::new_v4(),
                                binary: std::fs::read(binary)?,
                                machine_type: run.machine_type.clone(),
                            })
                        } else {
                            tracing::error!("No entry for bench {:?}, you should add an entry for it. If you dont want to run it in ussal set `framework` to `Skip`", artifact.filenames)
                        }
                    }
                }
            }
        }

        let output = command.wait().expect("Couldn't get cargo's exit status");
        if !output.success() {
            return Err(anyhow!("cargo build failed"));
        }

        if jobs.is_empty() {
            return Err(anyhow!("No benchmarks found"));
        }
    }

    Ok(jobs)
}
