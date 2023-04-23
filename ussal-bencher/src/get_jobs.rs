use crate::cli::Args;
use anyhow::{anyhow, Result};
use subprocess::{Exec, Redirection};
use ussal_shared::orchestrator_protocol::JobRequest;
use uuid::Uuid;

pub fn get_jobs(args: &Args) -> Result<Vec<JobRequest>> {
    // Run the command to stdout once so the user can see it.
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    if !std::process::Command::new(&cargo)
        .args(["bench", "--no-run"])
        .status()
        .unwrap()
        .success()
    {
        return Err(anyhow!("no"));
    }
    // Run the command again, this time capturing the output
    let output = run_command(&cargo, &["bench", "--no-run"])?;
    let mut jobs = vec![];
    for line in output.lines() {
        if line.trim().starts_with("Executable") {
            let mut iter = line.rsplit(|c| c == '(' || c == ')');
            iter.next()
                .ok_or_else(|| anyhow!("Unexpected 'Executable' line {line}"))?;
            let path = iter
                .next()
                .ok_or_else(|| anyhow!("Unexpected 'Executable' line {line}"))?;
            jobs.push(JobRequest {
                auth_token: args.auth_token,
                job_id: Uuid::new_v4(),
                binary: std::fs::read(path)?,
                os: "linux".to_owned(),
                arch: "x86_64".to_owned(),
            })
        }
    }

    Ok(jobs)
}

/// Runs a command and returns the output as a string.
/// Both stderr and stdout are returned in the result.
pub fn run_command(command: &str, args: &[&str]) -> Result<String> {
    let data = Exec::cmd(command)
        .args(args)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture()?;

    if data.exit_status.success() {
        Ok(data.stdout_str())
    } else {
        Err(anyhow!(
            "command {} {:?} exited with {:?} and output:\n{}",
            command,
            args,
            data.exit_status,
            data.stdout_str()
        ))
    }
}
