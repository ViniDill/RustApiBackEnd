use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateClientSchema {
    #[schema(example = "John Doe")]
    pub name: String,

    #[schema(example = "active")]
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateClientSchema {
    #[schema(example = "John Doe")]
    pub name: Option<String>,

    #[schema(example = "inactive")]
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateDeviceSchema {
    #[schema(example = "f47ac10b-58cc-4372-a567-0e02b2c3d479", format = "uuid")]
    pub client_id: String,

    #[schema(example = "123456789012345")]
    pub imei: String,

    #[schema(example = "Model X")]
    pub model: String,

    #[schema(example = "SN123456789")]
    pub serial_number: String,

    #[schema(example = "2025-07-18T12:34:56Z", format = "date-time")]
    pub upload_data: String,

    #[schema(example = "2025-07-18T12:35:56Z", format = "date-time")]
    pub upload_gps: String,

    #[schema(example = "active")]
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateDeviceSchema {
    #[schema(example = "Sensor Kitchen")]
    pub nickname: Option<String>,

    #[schema(example = "123456789012345")]
    pub imei: Option<String>,

    #[schema(example = "Model X")]
    pub model: Option<String>,

    #[schema(example = "2025-07-18T12:34:56Z", format = "date-time")]
    pub upload_data: Option<String>,

    #[schema(example = "2025-07-18T12:35:56Z", format = "date-time")]
    pub upload_gps: Option<String>,

    #[schema(example = "inactive")]
    pub status: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct FilterOptions {
    #[schema(example = 10)]
    pub limit: Option<usize>,

    #[schema(example = 1)]
    pub page: Option<usize>,
}
