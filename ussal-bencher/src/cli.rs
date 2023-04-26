use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Address of the ussal orchestrator web socket endpoint
    /// e.g. wss://some-ussal-instance.com/run_job
    #[clap(long, required = true)]
    pub address: String,

    /// Authorization token
    #[clap(long)]
    pub auth_token: uuid::Uuid,
}
