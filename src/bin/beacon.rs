use beaconship::lib::model::ShipAliveReq;
use clap::Clap;
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
    match result {
        Ok(resp) => match resp.status() {
            reqwest::StatusCode::OK => info!("pushover message sent"),
            code => warn!(
                "failed with statusCode {:?}, msg {:?}",
                code,
                resp.text().unwrap_or_else(|_| "can't read text".into())
            ),
        },
        Err(err) => warn!("request sent failed with error, {:?}", err),
    }
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
    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        loop {
            thread::sleep(Duration::from_secs(opts.interval));
            let mut ship_info = arc_thread.lock().unwrap();
            println!("ship_info: {:?}", ship_info);
            // ship_info.last_seens.iter_mut().for_each(|(k, v)| {});
            let mut ships_to_rm: Vec<String> = Vec::new();
            for (ship_id, last_seen) in ship_info.last_seens.iter() {
                let ship_req = ship_info.requests.get(ship_id).unwrap(); // guaranteed
                if *last_seen + Duration::from_secs(ship_req.max_offline) > SystemTime::now() {
                    ships_to_rm.push(ship_id.to_string());
                }
            }
            for ship_id in ships_to_rm.iter() {
                println!("removing sunk ship {}", ship_id);
                let last_seen = ship_info.last_seens.remove(ship_id).unwrap();
                let ship = ship_info.requests.remove(ship_id).unwrap();
                let msg = format!(
                    "Ship has sunk {} - last seen {:?}:\n\n{:?}",
                    ship.hostname, last_seen, ship
                );
                for user_token in opts.receivers.iter() {
                    send_notice(&msg, &opts.app_token, user_token, &client)
                }
            }
        }
    });
    rocket::build()
        .manage(arc)
        .mount("/", routes![hello, register_ship])
}
