use beaconship::model::ShipAliveReq;
use clap::Clap;
use futures::lock::Mutex;
use hyper::{client::HttpConnector, header, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use rocket::{response::status, serde::json::Json, State};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::Arc,
    thread,
    time::{Duration, SystemTime},
};
extern crate rocket;

const PUSHOVER_URL: &str = "https://api.pushover.net/1/messages.json";

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
type ClientType = hyper::Client<HttpsConnector<HttpConnector>>;
type ClientState = State<ClientType>;

pub fn make_client() -> ClientType {
    let https = HttpsConnector::new();
    hyper::Client::builder().build::<_, hyper::Body>(https)
}

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
#[derive(Serialize)]
struct PushOverMsg<'a> {
    message: &'a str,
    token: &'a str,
    user: &'a str,
}

async fn send_notice(msg: &str, app_token: &str, user_token: &str, client: &ClientType) {
    let body = serde_json::to_vec(&PushOverMsg {
        message: msg,
        token: app_token,
        user: user_token,
    })
    .unwrap();
    let req = Request::builder()
        .uri(PUSHOVER_URL)
        .header(header::CONTENT_TYPE, "application/json")
        .method(Method::POST)
        .body(body.into())
        .unwrap();
    let result = client.request(req).await;
    match result {
        Ok(resp) => match resp.status() {
            StatusCode::OK => info!("pushover message sent"),
            code => warn!("failed with statusCode {:?}, msg {:?}", code, *resp.body()),
        },
        Err(err) => warn!("request sent failed with error, {:?}", err),
    }
}

pub async fn check_sunk_ships(arc: Arc<Mutex<ShipInfoMap>>, opts: Arc<CmdOpts>) {
    let client = make_client();

    loop {
        thread::sleep(Duration::from_secs(opts.interval));
        let mut ship_info_map = arc.lock().await;
        debug!("ship_info_map: {:?}", ship_info_map);
        let ships_to_rm: Vec<String> = ship_info_map
            .iter()
            .filter_map(|(ship_id, ship_info)| {
                if ship_info.last_seen + Duration::from_secs(ship_info.request.max_offline)
                    < SystemTime::now()
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
            for user_token in opts.user_tokens.iter() {
                send_notice(&msg, &opts.app_token, user_token, &client).await
            }
        }
    }
}