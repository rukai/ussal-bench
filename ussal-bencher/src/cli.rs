use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Address of the ussal orchestrator
    #[clap(long, required = true)]
    pub address: String,

    /// Authorization token
    #[clap(long)]
    pub auth_token: uuid::Uuid,
}
