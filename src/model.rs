use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientModel {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceModel {
    pub id: Uuid,
    pub client_id: Uuid,
    pub nickname: String,
    pub imei: String,
    pub model: String,
    pub serial_number: String,
    pub upload_data: DateTime<Utc>,
    pub upload_gps: DateTime<Utc>,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}
