use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
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
