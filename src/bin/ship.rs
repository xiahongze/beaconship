//! Command line options
use clap::Clap;
use std::{thread, time::Duration};

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

fn main() {
    let opts = CmdOpts::parse();

    loop {
        thread::sleep(Duration::from_secs(opts.interval));
        println!("Hello, beacon!");
    }
}
