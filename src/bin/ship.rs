mod model;

use clap::Clap;
use model::ShipAliveReq;
use std::{thread, time::Duration};
// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Clap, Debug)]
pub struct CmdOpts {
    /// remote server address
    #[clap(short, long)]
    pub server: String,
    /// this hostname
    #[clap(short, long)]
    pub hostname: Option<String>,
    /// UUID of this ship
    #[clap(short, long)]
    pub uuid: Option<String>,
    /// heartbeat interval in seconds
    #[clap(short, long, default_value = "10")]
    pub interval: u64,
    /// maximum offline time in seconds at least 3x as large as interval
    #[clap(short, long, default_value = "30")]
    pub max_offline: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = CmdOpts::parse();
    let reqstruct = ShipAliveReq {
        hostname: opts.hostname.unwrap_or_else(|| "default".into()),
        max_offline: opts.max_offline,
        uuid: opts.uuid.unwrap_or_else(|| "default UUID".into()),
    };
    let client = reqwest::Client::new();

    loop {
        println!("Hello, beacon!");
        thread::sleep(Duration::from_secs(opts.interval));
        let url = reqwest::Url::parse(&opts.server).expect("Expect Legit URL");
        let body = serde_json::to_vec(&reqstruct).unwrap();
        let resp = client
            .post(url)
            .body(body)
            .header("content-type", "application/json")
            .send()
            .await?;
        match resp.status() {
            reqwest::StatusCode::OK => println!("success"),
            _ => println!("failed"),
        }
    }
}
