use serde::{Deserialize, Serialize};
// "Ship is alive" request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShipAliveReq {
    pub hostname: String,
    pub max_offline: u64,
    pub uuid: String,
}
