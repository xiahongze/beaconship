mod client;
mod pushover;
mod server;
use clap::Clap;
use client::make_client;
use futures::lock::Mutex;
use server::*;
use std::{
    sync::Arc,
    thread,
    time::{Duration, SystemTime},
};
#[macro_use]
extern crate rocket;

async fn check_sunk_ships(arc: Arc<Mutex<ShipInfoMap>>, opts: Arc<CmdOpts>) {
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
                pushover::send_notice(&msg, &opts.app_token, user_token, &client).await
            }
        }
    }
}

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let arc_opts = Arc::new(CmdOpts::parse());
    info!("{:?}", arc_opts);
    let arc_opts_thread = arc_opts.clone();

    let arc_ship = Arc::new(Mutex::new(ShipInfoMap::new()));
    let arc_ship_thread = arc_ship.clone();

    let rt = tokio::runtime::Runtime::new().unwrap();
    thread::spawn(move || rt.block_on(check_sunk_ships(arc_ship_thread, arc_opts_thread)));

    rocket::build()
        .manage(arc_ship)
        .manage(arc_opts)
        .manage(make_client())
        .mount("/", routes![get_ship, get_ships, del_ship, register_ship])
}
