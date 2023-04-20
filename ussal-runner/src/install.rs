use crate::cli::Args;
use anyhow::{anyhow, Result};
use subprocess::{Exec, Redirection};

pub fn install_runner(args: Args) {
    // Not a security check, just to provide a better error to the user.
    let user = run_command("whoami", &[]).unwrap().trim().to_string();
    if user != "root" {
        tracing::error!("This command will create user accounts, a systemd service and modify the sudoers file. So it needs to be run with root, but was instead run as the user '{user}'");
        return;
    }

    // Need to stop using the users in order to delete them
    run_command("systemctl", &["stop", "ussal-runner"]).ok();

    if user_exists("ussal-sandbox") {
        run_command("userdel", &["ussal-sandbox"]).unwrap();
    }
    run_command("useradd", &["ussal-sandbox"]).unwrap();

    if user_exists("ussal-runner") {
        // Purposefully do not delete the users home directory to avoid deleting certs and hitting letsencrypt's strict rate limits
        run_command("userdel", &["ussal-runner"]).unwrap();
    }
    run_command("useradd", &["-m", "ussal-runner"]).unwrap();
    std::fs::copy(
        std::env::current_exe().unwrap(),
        "/home/ussal-runner/ussal-runner",
    )
    .unwrap();
    run_command(
        "chown",
        &["-R", "ussal-runner:ussal-runner", "/home/ussal-runner"],
    )
    .unwrap();

    let email = args
        .email
        .map(|email| format!("--email {}", email))
        .unwrap_or("".to_owned());
    let domains = args.domains.join(" ");
    let start = format!("/home/ussal-runner/ussal-runner --mode orchestrator-and-runner --port 443 --domains {domains} {email}");

    let service_file = format!(
        r#"
[Unit]
Description={}
After=network.target
StartLimitIntervalSec=0

[Service]
AmbientCapabilities=CAP_NET_BIND_SERVICE
Type=simple
Restart=always
RestartSec=1
User=ussal-runner
ExecStart={}

[Install]
WantedBy=multi-user.target
"#,
        "Ussal orchestrator and runner", start
    );

    std::fs::write("/etc/systemd/system/ussal-runner.service", service_file).unwrap();
    run_command("systemctl", &["daemon-reload"]).unwrap();
    run_command("systemctl", &["enable", "ussal-runner"]).unwrap();
    run_command("systemctl", &["start", "ussal-runner"]).unwrap();
}

fn user_exists(name: &str) -> bool {
    std::fs::read_to_string("/etc/passwd")
        .unwrap()
        .lines()
        .any(|x| x.starts_with(&format!("{name}:")))
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
