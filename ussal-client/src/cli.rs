use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Address of the ussal orchestrator web socket endpoint
    /// e.g. wss://some-ussal-instance.com/run_job
    /// Overrides the address specified in the ussal.json
    #[clap(long)]
    pub address: Option<String>,

    /// Authorization token
    #[clap(long)]
    pub auth_token: uuid::Uuid,

    /// Authorization token
    #[clap(long)]
    pub ci: bool,

    /// Path to the ussal json config file.
    /// By default reads from `ussal.json`
    #[clap(long)]
    pub config_path: Option<String>,
}
