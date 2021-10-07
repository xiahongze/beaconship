use serde::{Deserialize, Serialize};

// "Ship is alive" request
#[derive(Debug, Serialize, Deserialize)]
pub struct ShipAliveReq {
    pub hostname: &str,
    pub max_offline: &u64,
    pub uuid: &str,
}
