use clap::Parser;

// TODO: make mode into a subcommand
#[derive(clap::ValueEnum, Clone, Copy)]
pub enum Mode {
    Runner,
    // TODO: put disable_https, domains and email in here
    Orchestrator,
    OrchestratorAndRunner,
    DestructivelyInstallRunner,
}

#[derive(Parser)]
pub struct Args {
    /// Operation mode for the runner.
    #[clap(long, required = true)]
    pub mode: Mode,

    /// Domains used in the letsencrypt certificate
    #[clap(long, required = true)]
    pub domains: Vec<String>,

    /// Email that letsencrypt will use to contact you if your certificate is failing to renew (indicates either a misconfiguration or a bug in ussal)
    #[clap(long)]
    pub email: Option<String>,

    /// The port the webserver will run on.
    /// Defaults to 443 when HTTPS is enabled or 8000 when HTTPS is disabled.
    #[clap(long)]
    pub port: Option<u16>,

    // TODO: make exclusive with email and domain flags
    /// Opens an HTTP port on port 8000 instead of an HTTPS port on port 443
    /// Normally the server binds to localhost and the external address.
    /// When this is enabled the server binds only to localhost to avoid exposing unencrypted communications over the network.
    ///
    /// This option is useful for a setup where you have another webserver such as nginx running on the same machine as the ussal-runner that provides https.
    #[clap(long)]
    pub disable_https: bool,
}
