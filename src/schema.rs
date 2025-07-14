use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateClientSchema {
    pub name: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateClientSchema {
    pub name: Option<String>,
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDeviceSchema {
    pub client_id: String,
    pub imei: String,
    pub model: String,
    pub serial_number: String,
    pub upload_data: String,
    pub upload_gps: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateDeviceSchema {
    pub nickname: Option<String>,
    pub imei: Option<String>,
    pub model: Option<String>,
    pub upload_data: Option<String>,
    pub upload_gps: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct FilterOptions {
    pub limit: Option<usize>,
    pub page: Option<usize>,
}
