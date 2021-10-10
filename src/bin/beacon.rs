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
    pub request: ShipAliveReq,
    pub last_seen: SystemTime,
}

type ShipInfoMap = HashMap<String, ShipInfo>;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[post("/ship", format = "application/json", data = "<ship>")]
fn register_ship(ship: Json<ShipAliveReq>, state: &State<Arc<Mutex<ShipInfoMap>>>) -> &'static str {
    let mut ship_info_map = state.lock().unwrap();
    let uuid = (*ship.uuid).to_string();
    if let std::collections::hash_map::Entry::Vacant(e) = ship_info_map.entry(uuid.clone()) {
        info!("Adding ship {:?} to the database", ship);
        e.insert(ShipInfo {
            request: (*ship).clone(),
            last_seen: SystemTime::now(),
        });
    } else {
        debug!("We have got {}", &uuid);
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

fn check_sunk_ships(arc: Arc<Mutex<ShipInfoMap>>, opts: CmdOpts) {
    let client = reqwest::blocking::Client::new();
    loop {
        thread::sleep(Duration::from_secs(opts.interval));
        let mut ship_info_map = arc.lock().unwrap();
        debug!("ship_info_map: {:?}", ship_info_map);
        let ships_to_rm: Vec<String> = ship_info_map
            .iter()
            .filter_map(|(ship_id, ship_info)| {
                if ship_info.last_seen + Duration::from_secs(ship_info.request.max_offline)
                    > SystemTime::now()
                {
                    Some(ship_id.clone())
                } else {
                    None
                }
            })
            .collect();

        for ship_id in ships_to_rm.iter() {
            info!("removing sunk ship {}", ship_id);
            let ship_info = ship_info_map.remove(ship_id).unwrap();
            let msg = format!(
                "Ship has sunk {} - last seen {:?}:\n\n{:?}",
                ship_info.request.hostname, ship_info.last_seen, ship_info
            );
            for user_token in opts.receivers.iter() {
                send_notice(&msg, &opts.app_token, user_token, &client)
            }
        }
    }
}

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let opts = CmdOpts::parse();
    info!("{:?}", opts);
    let arc = Arc::new(Mutex::new(ShipInfoMap::new()));

    let arc_thread = arc.clone();
    thread::spawn(move || check_sunk_ships(arc_thread, opts));

    rocket::build()
        .manage(arc)
        .mount("/", routes![hello, register_ship])
}
