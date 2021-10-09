mod model;
use model::ShipAliveReq;
use rocket::serde::json::Json;
#[macro_use]
extern crate rocket;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[post("/ship", format = "application/json", data = "<ship>")]
fn register_ship(ship: Json<ShipAliveReq>) -> &'static str {
    println!("ship: {:?}", ship);
    "ok"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, register_ship])
}
