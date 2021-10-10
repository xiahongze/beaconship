mod model;
use clap::Clap;
use model::ShipAliveReq;
use rocket::{serde::json::Json, State};
use std::{collections::HashMap, sync::Mutex, time::SystemTime};
#[macro_use]
extern crate rocket;

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

struct ShipInfo {
    pub requests: HashMap<String, ShipAliveReq>,
    pub last_seens: HashMap<String, SystemTime>,
}

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[post("/ship", format = "application/json", data = "<ship>")]
fn register_ship(ship: Json<ShipAliveReq>, state_data: &State<Mutex<ShipInfo>>) -> &'static str {
    println!("ship: {:?}", ship);
    let mut ship_info = state_data.lock().unwrap();
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

#[launch]
fn rocket() -> _ {
    let opts = CmdOpts::parse();
    println!("{:?}", opts);
    rocket::build()
        .manage(Mutex::new(ShipInfo {
            requests: HashMap::new(),
            last_seens: HashMap::new(),
        }))
        .mount("/", routes![hello, register_ship])
}
