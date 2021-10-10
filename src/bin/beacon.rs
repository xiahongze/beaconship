mod model;
use clap::Clap;
use model::ShipAliveReq;
use rocket::{serde::json::Json, State};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};
#[macro_use]
extern crate rocket;

const PUSHOVER_URL: &str = "https://api.pushover.net/1/messages.json";

#[derive(Clap, Debug)]
struct CmdOpts {
    /// PushOver App Token
    #[clap(short, long)]
    pub app_token: String,
    /// list of receivers to notify (PushOver User Tokens)
    #[clap(short, long)]
    pub receivers: Vec<String>,
    /// interval in seconds to scan for sunk ships
    #[clap(short, long, default_value = "5")]
    pub interval: u64,
}

#[derive(Debug)]
struct ShipInfo {
    pub requests: HashMap<String, ShipAliveReq>,
    pub last_seens: HashMap<String, SystemTime>,
}

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[post("/ship", format = "application/json", data = "<ship>")]
fn register_ship(ship: Json<ShipAliveReq>, state: &State<Arc<Mutex<ShipInfo>>>) -> &'static str {
    println!("ship: {:?}", ship);
    let mut ship_info = state.lock().unwrap();
    let uuid = (*ship.uuid).to_string();
    if ship_info.requests.contains_key(&uuid) {
        println!("We have got {}", &uuid);
    } else {
        println!("Adding ship {:?} to the database", ship);
        ship_info.requests.insert(uuid.clone(), (*ship).clone());
        ship_info.last_seens.insert(uuid, SystemTime::now());
    }
    "ok"
}
#[derive(Serialize)]
struct PushOverMsg<'a> {
    message: &'a str,
    token: &'a str,
    user: &'a str,
}

fn send_notice(msg: &str, app_token: &str, user_token: &str, client: &reqwest::blocking::Client) {
    let result = client
        .post(PUSHOVER_URL)
        .header("content-type", "application/json")
        .body(
            serde_json::to_vec(&PushOverMsg {
                message: msg,
                token: app_token,
                user: user_token,
            })
            .unwrap(),
        )
        .send();
}

#[launch]
fn rocket() -> _ {
    let opts = CmdOpts::parse();
    println!("{:?}", opts);
    let arc = Arc::new(Mutex::new(ShipInfo {
        requests: HashMap::new(),
        last_seens: HashMap::new(),
    }));

    let arc_thread = arc.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(opts.interval));
        let mut state = arc_thread.lock().unwrap();
        println!("state: {:?}", state);
    });
    rocket::build()
        .manage(arc)
        .mount("/", routes![hello, register_ship])
}
