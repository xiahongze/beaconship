mod model;

use clap::Clap;
use hyper::{Body, Client, Request};
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
        hostname: opts.hostname.unwrap_or("default".into()),
        max_offline: opts.max_offline,
        uuid: opts.uuid.unwrap_or("default UUID".into()),
    };
    let client = Client::new();
    // let uri = opts.server.parse::<hyper::Uri>().unwrap();
    // let req_body = Body::from(serde_json::to_vec(&reqstruct).unwrap());
    // let req = Request::builder()
    //     .method("POST")
    //     .uri(uri)
    //     .body(req_body)
    //     .unwrap();

    loop {
        thread::sleep(Duration::from_secs(opts.interval));
        println!("Hello, beacon!");
        let uri = opts.server.parse::<hyper::Uri>().unwrap();
        let req_body = Body::from(serde_json::to_vec(&reqstruct).unwrap());
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .body(req_body)
            .unwrap();
        let mut res = client.request(req);
    }
}
