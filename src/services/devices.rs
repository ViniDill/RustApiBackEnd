use actix_web::{
    web::{Json, Path, Data, Query},
    get, post, patch, delete, HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;
#[allow(unused_imports)]
use utoipa::ToSchema;

use crate::{
    schema::{CreateDeviceSchema, FilterOptions, UpdateDeviceSchema},
    model::{ClientModel, DeviceModel},
    AppState,
};

#[utoipa::path(
    request_body = CreateDeviceSchema,
    responses(
        (status = 200, description = "Create a new device.", body = DeviceModel),
        (status = 400, description = "Invalid client_id UUID"),
        (status = 404, description = "Client not found"),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Dispositivos"
)]
#[post("/devices")]
pub async fn create_device(
    body: Json<CreateDeviceSchema>,
    data: Data<AppState>
) -> impl Responder {
    let client_id = match Uuid::parse_str(&body.client_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!( {
                "status": "error",
                "message": "Invalid client_id UUID"
            }));
        }
    };

    let client = match sqlx::query_as!(
        ClientModel,
        "SELECT * FROM clients WHERE id = $1",
        client_id
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!( {
                "status": "error",
                "message": "Client not found"
            }));
        }
    };

    let nickname = format!("{}{}", client.name.to_lowercase(), body.serial_number);

    let upload_data = match body.upload_data.parse::<chrono::DateTime<chrono::Utc>>() {
        Ok(dt) => dt,
        Err(_) => return HttpResponse::BadRequest().json(json!( {
            "status": "error",
            "message": "Invalid upload_data datetime format"
        })),
    };

    let upload_gps = match body.upload_gps.parse::<chrono::DateTime<chrono::Utc>>() {
        Ok(dt) => dt,
        Err(_) => return HttpResponse::BadRequest().json(json!( {
            "status": "error",
            "message": "Invalid upload_gps datetime format"
        })),
    };

    match sqlx::query_as!(
        DeviceModel,
        r#"
        INSERT INTO devices 
            (client_id, nickname, imei, model, serial_number, upload_data, upload_gps, status)
        VALUES 
            ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
        client_id,
        nickname,
        body.imei,
        body.model,
        body.serial_number,
        upload_data,
        upload_gps,
        body.status
    )
    .fetch_one(&data.db)
    .await {
        Ok(device) => HttpResponse::Ok().json(json!( {
            "status": "success",
            "device": device,
        })),
        Err(error) => HttpResponse::InternalServerError().json(json!( {
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "List all devices.", body = [DeviceModel]),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Dispositivos"
)]
#[get("/devices")]
pub async fn get_all_devices(
    opts: Query<FilterOptions>,
    data: Data<AppState>
) -> impl Responder {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    match sqlx::query_as!(
        DeviceModel,
        "SELECT * FROM devices ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await {
        Ok(devices) => HttpResponse::Ok().json(json!( {
            "status": "success",
            "result": devices.len(),
            "devices": devices,
        })),
        Err(error) => HttpResponse::InternalServerError().json(json!( {
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Get device by ID.", body = DeviceModel),
        (status = 404, description = "Device not found"),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Dispositivos"
)]
#[get("/devices/{id}")]
pub async fn get_device_by_id(
    path: Path<Uuid>,
    data: Data<AppState>
) -> impl Responder {
    let device_id = path.into_inner();

    match sqlx::query_as!(
        DeviceModel,
        "SELECT * FROM devices WHERE id = $1",
        device_id
    )
    .fetch_one(&data.db)
    .await {
        Ok(device) => HttpResponse::Ok().json(json!( {
            "status": "success",
            "device": device,
        })),
        Err(error) => HttpResponse::NotFound().json(json!( {
            "status": "not found",
            "message": format!("{:?}", error)
        })),
    }
}

#[utoipa::path(
    request_body = UpdateDeviceSchema,
    responses(
        (status = 200, description = "Update device by ID.", body = DeviceModel),
        (status = 404, description = "Device not found"),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Dispositivos"
)]
#[patch("/devices/{id}")]
pub async fn update_device_by_id(
    path: Path<Uuid>,
    body: Json<UpdateDeviceSchema>,
    data: Data<AppState>
) -> impl Responder {
    let device_id = path.into_inner();

    let existing_device = sqlx::query_as!(
        DeviceModel,
        "SELECT * FROM devices WHERE id = $1",
        device_id
    )
    .fetch_one(&data.db)
    .await;

    if let Err(error) = existing_device {
        return HttpResponse::NotFound().json(json!( {
            "status": "not found",
            "message": format!("{:?}", error)
        }));
    }

    let device = existing_device.unwrap();

    let upload_data = body.upload_data.as_ref()
        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
        .unwrap_or(device.upload_data);

    let upload_gps = body.upload_gps.as_ref()
        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
        .unwrap_or(device.upload_gps);

    match sqlx::query_as!(
        DeviceModel,
        "UPDATE devices SET nickname = $1, imei = $2, model = $3, upload_data = $4, upload_gps = $5, status = $6 WHERE id = $7 RETURNING *",
        body.nickname.clone().unwrap_or(device.nickname),
        body.imei.clone().unwrap_or(device.imei),
        body.model.clone().unwrap_or(device.model),
        upload_data,
        upload_gps,
        body.status.clone().unwrap_or(device.status),
        device_id
    )
    .fetch_one(&data.db)
    .await {
        Ok(updated_device) => HttpResponse::Ok().json(json!( {
            "status": "success",
            "device": updated_device,
        })),
        Err(error) => HttpResponse::InternalServerError().json(json!( {
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[utoipa::path(
    responses(
        (status = 204, description = "Delete device by ID."),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Dispositivos"
)]
#[delete("/devices/{id}")]
pub async fn delete_device_by_id(
    path: Path<Uuid>,
    data: Data<AppState>
) -> impl Responder {
    let device_id = path.into_inner();

    match sqlx::query!(
        "DELETE FROM devices WHERE id = $1",
        device_id
    )
    .execute(&data.db)
    .await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(error) => HttpResponse::InternalServerError().json(json!( {
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}
