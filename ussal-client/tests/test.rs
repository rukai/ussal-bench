#[cfg(not(target_os = "windows"))]
mod test {
    use serial_test::serial;
    use std::time::Duration;
    use subprocess::{Exec, Redirection};
    use tokio_bin_process::event::Level;
    use tokio_bin_process::event_matcher::EventMatcher;
    use tokio_bin_process::BinProcess;

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_invalid_auth_key() {
        let runner = ussal_server().await;

        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let output = run_failing_command(
            &cargo,
            &[
                "run",
                "-p",
                "ussal-client",
                "--",
                "--config-path",
                "ussal-test.json",
                "--auth-token",
                "deaddead-dead-dead-dead-deaddeaddead",
            ],
        );
        assert!(
            output.contains("Failed to run remote benchmarks: Invalid auth token"),
            "ussal-client did not contain expected output, was instead:\n{output}"
        );

        runner.shutdown_and_then_consume_events(&[]).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_success() {
        let runner = ussal_server().await;

        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let output = run_command(
            &cargo,
            &[
                "run",
                "-p",
                "ussal-client",
                "--",
                "--config-path",
                "ussal-test.json",
                "--auth-token",
                "2d58efc6-6c95-47c5-968d-55aa923b4cc9",
            ],
        );
        assert!(
            // TODO: improve the output of the client and then assert on it
            output.contains("BenchComplete { bench_name:"),
            "ussal-client did not contain expected output, was instead:\n{output}"
        );

        runner.shutdown_and_then_consume_events(&[]).await;
    }

    async fn ussal_server() -> BinProcess {
        let mut runner = BinProcess::start_binary_name(
            "ussal-server",
            "server",
            &[
                "--log-format",
                "json",
                "--config-path",
                "tests/server-config",
                "--sandbox-mode",
                "none",
                "orchestrator-and-runner",
                "--disable-https",
                "--domains",
                "deletethis",
            ],
            None,
        )
        .await;

        tokio::time::timeout(
            Duration::from_secs(30),
            runner.wait_for(
                &EventMatcher::new()
                    .with_level(Level::Info)
                    .with_target("ussal_server")
                    .with_message("Starting HTTP on port: 8000"),
                &[],
            ),
        )
        .await
        .unwrap();
        runner
    }

    /// Runs a command and returns the output as a string.
    /// Both stderr and stdout are returned in the result.
    fn run_command(command: &str, args: &[&str]) -> String {
        let data = Exec::cmd(command)
            .args(args)
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Merge)
            .capture()
            .unwrap();

        if data.exit_status.success() {
            data.stdout_str()
        } else {
            panic!(
                "command {} {:?} exited with {:?} and output:\n{}",
                command,
                args,
                data.exit_status,
                data.stdout_str()
            )
        }
    }

    /// Runs a command asserting that it failed and returns the output as a string.
    /// Both stderr and stdout are returned in the result.
    fn run_failing_command(command: &str, args: &[&str]) -> String {
        let data = Exec::cmd(command)
            .args(args)
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Merge)
            .capture()
            .unwrap();

        if !data.exit_status.success() {
            data.stdout_str()
        } else {
            panic!(
                "Expected command command {} {:?} to fail. But it exited with {:?} and output:\n{}",
                command,
                args,
                data.exit_status,
                data.stdout_str()
            )
        }
    }
}
