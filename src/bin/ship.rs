use beaconship::lib::model::ShipAliveReq;
use clap::Clap;
use std::{thread, time::Duration};
// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[macro_use]
extern crate log;

#[derive(Clap, Debug)]
struct CmdOpts {
    /// remote server address
    #[clap(short, long)]
    pub server: String,
    /// this hostname, default to os hostname
    #[clap(short, long)]
    pub hostname: Option<String>,
    /// heartbeat interval in seconds
    #[clap(short, long, default_value = "10")]
    pub interval: u64,
    /// maximum offline time in seconds at least 3x as large as interval
    #[clap(short, long, default_value = "30")]
    pub max_offline: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let opts = CmdOpts::parse();

    let reqstruct = ShipAliveReq {
        hostname: opts.hostname.unwrap_or_else(|| {
            hostname::get()
                .unwrap_or_else(|_| "unknown host".into())
                .into_string()
                .unwrap_or_else(|_| "unknown host".into())
        }),
        max_offline: opts.max_offline,
        uuid: uuid::Uuid::new_v4().to_string(),
    };
    info!("Ship sailed with {:?}", reqstruct);

    let client = reqwest::Client::new();

    loop {
        let url = reqwest::Url::parse(&opts.server).expect("Expect Legit URL");
        let body = serde_json::to_vec(&reqstruct).unwrap();
        let result = client
            .post(url)
            .body(body)
            .header("content-type", "application/json")
            .send()
            .await;
        match result {
            Ok(resp) => match resp.status() {
                reqwest::StatusCode::OK => info!("success"),
                code => warn!(
                    "failed with statusCode {:?}, msg {:?}",
                    code,
                    resp.text()
                        .await
                        .unwrap_or_else(|_| "can't read text".into())
                ),
            },
            Err(err) => warn!("request sent failed with error, {:?}", err),
        }
        thread::sleep(Duration::from_secs(opts.interval));
    }
}
