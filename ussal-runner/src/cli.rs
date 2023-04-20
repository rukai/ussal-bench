use clap::Parser;

// TODO: make mode into a subcommand
#[derive(clap::ValueEnum, Clone, Copy)]
pub enum Mode {
    Runner,
    // TODO: put domains and email in here
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
    #[clap(long, default_value = "1443")]
    pub port: u16,
}
