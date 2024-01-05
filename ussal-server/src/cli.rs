use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Mode {
    Runner {
        /// The address of the orchestrator to take requests from
        #[clap(long, required = true)]
        address: String,

        /// An arbitrary string containing the machine-type of this runner, the runner will only receive jobs that request this machine-type
        #[clap(long, required = true)]
        machine_type: String,
    },
    Orchestrator {
        /// Domains used in the letsencrypt certificate
        #[clap(long, required = true)]
        domains: Vec<String>,

        /// Email that letsencrypt will use to contact you if your certificate is failing to renew (indicates either a misconfiguration or a bug in ussal)
        #[clap(long)]
        email: Option<String>,

        /// The port the webserver will run on.
        /// Defaults to 443 when HTTPS is enabled or 8000 when HTTPS is disabled.
        #[clap(long)]
        port: Option<u16>,

        /// Opens an HTTP port on port 8000 instead of an HTTPS port on port 443
        /// Normally the server binds to localhost and the external address.
        /// When this is enabled the server binds only to localhost to avoid exposing unencrypted communications over the network.
        ///
        /// This option is useful for a setup where you have another webserver such as nginx running on the same machine as the ussal-server that provides https.
        #[clap(long)]
        disable_https: bool,
    },
    OrchestratorAndRunner {
        /// Domains used in the letsencrypt certificate
        #[clap(long, required = true)]
        domains: Vec<String>,

        /// Email that letsencrypt will use to contact you if your certificate is failing to renew (indicates either a misconfiguration or a bug in ussal)
        #[clap(long)]
        email: Option<String>,

        /// The port the webserver will run on.
        /// Defaults to 443 when HTTPS is enabled or 8000 when HTTPS is disabled.
        #[clap(long)]
        port: Option<u16>,

        /// Opens an HTTP port on port 8000 instead of an HTTPS port on port 443
        /// Normally the server binds to localhost and the external address.
        /// When this is enabled the server binds only to localhost to avoid exposing unencrypted communications over the network.
        ///
        /// This option is useful for a setup where you have another webserver such as nginx running on the same machine as the ussal-server that provides https.
        #[clap(long)]
        disable_https: bool,
    },
    /// Install or reinstall the ussal runner to this machine.
    ///
    /// Ussal is setup as an enabled and started systemd unit named `ussal-server`.
    /// Existing ussal config and letsencrypt state is retained.
    DestructivelyInstallRunner {
        /// Domains used in the letsencrypt certificate
        #[clap(long, required = true)]
        domains: Vec<String>,

        /// Email that letsencrypt will use to contact you if your certificate is failing to renew (indicates either a misconfiguration or a bug in ussal)
        #[clap(long)]
        email: Option<String>,
    },
}

pub struct OrchestratorArgs {
    pub domains: Vec<String>,
    pub email: Option<String>,
    pub port: Option<u16>,
    pub disable_https: bool,
}

impl Mode {
    pub fn orchestrator_args(&self) -> OrchestratorArgs {
        match self {
            Mode::Orchestrator {
                domains,
                email,
                port,
                disable_https,
            }
            | Mode::OrchestratorAndRunner {
                domains,
                email,
                port,
                disable_https,
            } => OrchestratorArgs {
                domains: domains.clone(),
                email: email.clone(),
                port: *port,
                disable_https: *disable_https,
            },
            Mode::DestructivelyInstallRunner { email, domains } => OrchestratorArgs {
                email: email.clone(),
                domains: domains.clone(),
                port: None,
                disable_https: false,
            },
            _ => unreachable!("This must only be called when it is known to use orchestrator args"),
        }
    }
}

#[derive(Parser)]
pub struct Args {
    /// Operation mode for the runner.
    #[command(subcommand)]
    pub mode: Mode,

    #[clap(long, value_enum, default_value = "human", verbatim_doc_comment)]
    pub log_format: LogFormat,

    /// Path to the ussal config directory.
    /// By default reads from `~/.config/UssalRunner/` where `~` is likely `/home/ussal-server`.
    #[clap(long, verbatim_doc_comment)]
    pub config_path: Option<String>,

    /// Level of sandboxing to use when running bench executables
    #[clap(
        long,
        value_enum,
        default_value = "nsjail-complete",
        verbatim_doc_comment
    )]
    // TODO: should this be part of the config file instead?
    pub sandbox_mode: SandboxMode,
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub enum LogFormat {
    Human,
    Json,
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub enum SandboxMode {
    /// Complete nsjail sandboxing.
    /// Only the most basic syscalls for program execution are allowed. Things like networking and disk are inaccessible.
    NsjailComplete,
    /// Never expose this to the internet
    None,
}
