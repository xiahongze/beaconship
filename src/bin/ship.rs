use beaconship::model::ShipAliveReq;
use clap::Clap;
use std::{thread, time::Duration};
// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[macro_use]
extern crate log;
use hyper::{header, Method, Request, StatusCode, Uri};

#[derive(Clap, Debug)]
/// Ship reports to Beacon if not sunk
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
    #[clap(short, long, default_value = "60")]
    pub max_offline: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

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

    let client = hyper::Client::new();

    loop {
        let uri = opts.server.parse::<Uri>().expect("Expect a Legit URI");
        let body = serde_json::to_vec(&reqstruct).unwrap();
        let req = Request::builder()
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .method(Method::POST)
            .body(body.into())
            .unwrap();
        let result = client.request(req).await;
        match result {
            Ok(resp) => match resp.status() {
                StatusCode::OK => debug!("success"),
                code => warn!("failed with statusCode {:?}, msg {:?}", code, *resp.body()),
            },
            Err(err) => warn!("request sent failed with error, {:?}", err),
        }
        thread::sleep(Duration::from_secs(opts.interval));
    }
}
