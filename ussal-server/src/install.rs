use crate::cli::Args;
use crate::system::run_command;
use std::fs::OpenOptions;
use std::io::Write;

pub fn install_runner(args: Args) {
    // Not a security check, just to provide a better error to the user.
    let user = run_command("whoami", &[]).unwrap().trim().to_string();
    if user != "root" {
        tracing::error!("This command will create user accounts, a systemd service and modify the sudoers file. So it needs to be run with root, but was instead run as the user '{user}'");
        return;
    }

    // Need to stop using the users in order to delete them
    run_command("systemctl", &["stop", "ussal-server"]).ok();

    if user_exists("ussal-sandbox") {
        run_command("userdel", &["ussal-sandbox"]).unwrap();
    }
    run_command("useradd", &["ussal-sandbox"]).unwrap();

    if user_exists("ussal-server") {
        // Purposefully do not delete the users home directory to avoid deleting certs and hitting letsencrypt's strict rate limits
        run_command("userdel", &["ussal-server"]).unwrap();
    }
    run_command("useradd", &["-m", "ussal-server"]).unwrap();
    std::fs::copy(
        std::env::current_exe().unwrap(),
        "/home/ussal-server/ussal-server",
    )
    .unwrap();
    run_command(
        "chown",
        &["-R", "ussal-server:ussal-server", "/home/ussal-server"],
    )
    .unwrap();

    let args = args.mode.orchestrator_args();
    let email = args
        .email
        .map(|email| format!("--email {}", email))
        .unwrap_or("".to_owned());
    let domains = args.domains.join(" ");
    let start = format!("/home/ussal-server/ussal-server --port 443 --mode orchestrator-and-runner --domains {domains} {email}");

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
User=ussal-server
ExecStart={}

[Install]
WantedBy=multi-user.target
"#,
        "Ussal orchestrator and runner", start
    );

    if !std::fs::read_to_string("/etc/sudoers")
        .unwrap()
        .contains("ussal-server")
    {
        let mut sudoers = OpenOptions::new()
            .append(true)
            .open("/etc/sudoers")
            .unwrap();
        writeln!(
            sudoers,
            "\nussal-server ALL = (ussal-sandbox) NOPASSWD: ALL"
        )
        .unwrap();
    }

    std::fs::write("/etc/systemd/system/ussal-server.service", service_file).unwrap();
    run_command("systemctl", &["daemon-reload"]).unwrap();
    run_command("systemctl", &["enable", "ussal-server"]).unwrap();
    run_command("systemctl", &["start", "ussal-server"]).unwrap();
}

fn user_exists(name: &str) -> bool {
    std::fs::read_to_string("/etc/passwd")
        .unwrap()
        .lines()
        .any(|x| x.starts_with(&format!("{name}:")))
}
