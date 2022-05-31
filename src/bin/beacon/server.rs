use beaconship::model::ShipAliveReq;
use clap::Clap;
use futures::lock::Mutex;

use rocket::{response::status, serde::json::Json, State};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc, time::SystemTime};

use crate::{client::ClientType, pushover::send_notice};
extern crate rocket;

#[derive(Clap, Debug)]
/// Beacon stands as the watcher of ships
pub struct CmdOpts {
    /// PushOver App Token
    #[clap(short, long, env)]
    pub app_token: String,
    /// list of receivers to notify (PushOver User Tokens)
    #[clap(short, long, env)]
    pub user_tokens: Vec<String>,
    /// interval in seconds to scan for sunk ships
    #[clap(short, long, default_value = "5")]
    pub interval: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ShipInfo {
    pub request: ShipAliveReq,
    pub last_seen: SystemTime,
}
pub type ShipInfoMap = HashMap<String, ShipInfo>;
type ShipState = State<Arc<Mutex<ShipInfoMap>>>;
type ClientState = State<ClientType>;

#[get("/ship/<uuid>")]
pub async fn get_ship(
    uuid: &str,
    state: &ShipState,
) -> Result<Json<ShipInfo>, status::NotFound<String>> {
    let ship_info_map = state.lock().await;
    ship_info_map
        .get(uuid)
        .map(|ship_info| Ok(Json(ship_info.clone())))
        .unwrap_or_else(|| Err(status::NotFound(format!("Ship ({}) not found", uuid))))
}

#[get("/ship/list")]
pub async fn get_ships(state: &ShipState) -> Json<Vec<ShipInfo>> {
    let ship_info_map = state.lock().await;
    Json(ship_info_map.values().cloned().collect())
}

#[delete("/ship/<uuid>")]
pub async fn del_ship(
    uuid: &str,
    state: &ShipState,
) -> Result<&'static str, status::NotFound<String>> {
    let mut ship_info_map = state.lock().await;
    ship_info_map
        .remove(uuid)
        .map(|ship_info| {
            info!("Ship ({:?}) removed", ship_info);
            Ok("ok")
        })
        .unwrap_or_else(|| Err(status::NotFound(format!("Ship ({}) not found", uuid))))
}

#[post("/ship", format = "application/json", data = "<ship>")]
pub async fn register_ship(
    ship: Json<ShipAliveReq>,
    ship_state: &ShipState,
    client: &ClientState,
    opts: &State<Arc<CmdOpts>>,
) -> &'static str {
    let mut ship_info_map = ship_state.lock().await;
    let uuid = (*ship.uuid).to_string();
    match ship_info_map.get_mut(&uuid) {
        Some(ship_info) => ship_info.last_seen = SystemTime::now(),
        None => {
            info!("Ship has registered. {:?}", ship);
            let msg = format!("Ship {} has registered.\n{:?}", ship.hostname, ship);
            for user in opts.user_tokens.iter() {
                send_notice(&msg, &opts.app_token, user, client).await;
            }
            ship_info_map.insert(
                uuid.clone(),
                ShipInfo {
                    request: (*ship).clone(),
                    last_seen: SystemTime::now(),
                },
            );
        }
    }
    "ok"
}
