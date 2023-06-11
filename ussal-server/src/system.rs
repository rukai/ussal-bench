use crate::cli::LogFormat;
use anyhow::{anyhow, Result};
use subprocess::{Exec, Redirection};
use tracing_appender::non_blocking::WorkerGuard;

/// Runs a command and returns the output as a string.
/// Both stderr and stdout are returned in the result.
pub fn run_command(command: &str, args: &[&str]) -> Result<String> {
    let data = Exec::cmd(command)
        .args(args)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture()
        .map_err(|e| anyhow!(e).context(format!("Failed to run command {} {:?}", command, args)))?;

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

/// Runs a binary in an nsjail and returns the output as a string.
/// Both stderr and stdout are returned in the result.
pub fn run_sandboxed_binary(command: &str, args: &[&str]) -> Result<String> {
    let mut nsjail_args = vec![
        "--really_quiet",
        "--mode",
        "o",
        "--user",
        "99999",
        "--group",
        "99999",
        "--keep_caps",
        "-R",
        "/usr/lib",
        "-R",
        "/lib",
        "-R",
        "/dev/urandom",
        "-R",
        "/home/ussal-server/binary-under-test",
        "--",
        command,
    ];
    nsjail_args.extend(args);
    run_command("nsjail", &nsjail_args)
}

pub fn init_tracing(format: LogFormat) -> WorkerGuard {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

    let builder = tracing_subscriber::fmt().with_writer(non_blocking);

    match format {
        LogFormat::Json => builder.json().init(),
        LogFormat::Human => builder.init(),
    }

    // When in json mode we need to process panics as events instead of printing directly to stdout.
    // This is so that:
    // * We dont include invalid json in stdout
    // * panics can be received by whatever is processing the json events
    //
    // We dont do this for LogFormat::Human because the default panic messages are more readable for humans
    if let LogFormat::Json = format {
        crate::tracing_panic_handler::setup();
    }

    guard
}

#[cfg(target_os = "windows")]
pub async fn init_shutdown_handler() -> tokio::sync::watch::Receiver<bool> {
    use tokio::sync::watch;
    let (trigger_shutdown_tx, trigger_shutdown_rx) = watch::channel(false);
    std::mem::forget(trigger_shutdown_tx);
    trigger_shutdown_rx
}

#[cfg(not(target_os = "windows"))]
pub async fn init_shutdown_handler() -> tokio::sync::watch::Receiver<bool> {
    use tokio::{
        signal::unix::{signal, SignalKind},
        sync::watch,
    };
    // We need to block on this part to ensure that we immediately register these signals.
    // Otherwise if we included signal creation in the below spawned task we would be at the mercy of whenever tokio decides to start running the task.
    let mut interrupt = signal(SignalKind::interrupt()).unwrap();
    let mut terminate = signal(SignalKind::terminate()).unwrap();
    let (trigger_shutdown_tx, trigger_shutdown_rx) = watch::channel(false);
    tokio::spawn(async move {
        tokio::select! {
            _ = interrupt.recv() => {
                tracing::info!("shutting down from SIGINT");
            },
            _ = terminate.recv() => {
                tracing::info!("shutting down from SIGTERM");
            },
        };

        trigger_shutdown_tx.send(true).unwrap();
    });

    trigger_shutdown_rx
}
