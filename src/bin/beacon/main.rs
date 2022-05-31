mod server;
use clap::Clap;
use futures::lock::Mutex;
use server::*;
use std::{sync::Arc, thread};
#[macro_use]
extern crate rocket;

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
