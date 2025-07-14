use actix_web::{
    web::{scope, Json, Path, Data, ServiceConfig, Query},
    get, post, delete, patch, HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    schema::{CreateClientSchema, CreateDeviceSchema, FilterOptions, UpdateClientSchema, UpdateDeviceSchema},
    model::{ClientModel, DeviceModel},
    AppState,
};

#[get("/healthchecker")]
async fn health_checker() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Health check: API is up and running smoothly."
    }))
}

#[post("/clients")]
async fn create_client(
    body: Json<CreateClientSchema>,
    data: Data<AppState>
) -> impl Responder {
    match sqlx::query_as!(
        ClientModel,
        r#"
        INSERT INTO clients (name, status)
        VALUES ($1, $2)
        RETURNING *
        "#,
        body.name,
        body.status
    )
    .fetch_one(&data.db)
    .await {
        Ok(client) => HttpResponse::Ok().json(json!({
            "status": "success",
            "client": client,
        })),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[get("/clients")]
async fn get_all_clients(
    opts: Query<FilterOptions>,
    data: Data<AppState>
) -> impl Responder {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    match sqlx::query_as!(
        ClientModel,
        "SELECT * FROM clients ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await {
        Ok(clients) => HttpResponse::Ok().json(json!({
            "status": "success",
            "result": clients.len(),
            "clients": clients,
        })),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[get("/clients/{id}")]
async fn get_client_by_id(
    path: Path<Uuid>,
    data: Data<AppState>
) -> impl Responder {
    let client_id = path.into_inner();

    match sqlx::query_as!(
        ClientModel,
        "SELECT * FROM clients WHERE id = $1",
        client_id
    )
    .fetch_one(&data.db)
    .await {
        Ok(client) => HttpResponse::Ok().json(json!({
            "status": "success",
            "client": client,
        })),
        Err(error) => HttpResponse::NotFound().json(json!({
            "status": "not found",
            "message": format!("{:?}", error)
        })),
    }
}

#[patch("/clients/{id}")]
async fn update_client_by_id(
    path: Path<Uuid>,
    body: Json<UpdateClientSchema>,
    data: Data<AppState>
) -> impl Responder {
    let client_id = path.into_inner();

    let existing_client = sqlx::query_as!(
        ClientModel,
        "SELECT * FROM clients WHERE id = $1",
        client_id
    )
    .fetch_one(&data.db)
    .await;

    if let Err(error) = existing_client {
        return HttpResponse::NotFound().json(json!({
            "status": "not found",
            "message": format!("{:?}", error)
        }));
    }

    let client = existing_client.unwrap();

    match sqlx::query_as!(
        ClientModel,
        "UPDATE clients SET name = $1, status = $2 WHERE id = $3 RETURNING *",
        body.name.clone().unwrap_or(client.name),
        body.status.clone().unwrap_or(client.status),
        client_id
    )
    .fetch_one(&data.db)
    .await {
        Ok(updated_client) => HttpResponse::Ok().json(json!({
            "status": "success",
            "client": updated_client,
        })),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[delete("/clients/{id}")]
async fn delete_client_by_id(
    path: Path<Uuid>,
    data: Data<AppState>
) -> impl Responder {
    let client_id = path.into_inner();

    match sqlx::query!(
        "DELETE FROM clients WHERE id = $1",
        client_id
    )
    .execute(&data.db)
    .await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[post("/devices")]
async fn create_device(
    body: Json<CreateDeviceSchema>,
    data: Data<AppState>
) -> impl Responder {
    let client_id = match Uuid::parse_str(&body.client_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
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
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Cliente não encontrado"
            }));
        }
    };

    let nickname = format!("{}{}", client.name.to_lowercase(), body.serial_number);

    let upload_data = match body.upload_data.parse::<chrono::DateTime<chrono::Utc>>() {
        Ok(dt) => dt,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid upload_data datetime format"
        })),
    };

    let upload_gps = match body.upload_gps.parse::<chrono::DateTime<chrono::Utc>>() {
        Ok(dt) => dt,
        Err(_) => return HttpResponse::BadRequest().json(json!({
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
        Ok(device) => HttpResponse::Ok().json(json!({
            "status": "success",
            "device": device,
        })),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[get("/devices")]
async fn get_all_devices(
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
        Ok(devices) => HttpResponse::Ok().json(json!({
            "status": "success",
            "result": devices.len(),
            "devices": devices,
        })),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[get("/devices/{id}")]
async fn get_device_by_id(
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
        Ok(device) => HttpResponse::Ok().json(json!({
            "status": "success",
            "device": device,
        })),
        Err(error) => HttpResponse::NotFound().json(json!({
            "status": "not found",
            "message": format!("{:?}", error)
        })),
    }
}

#[patch("/devices/{id}")]
async fn update_device_by_id(
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
        return HttpResponse::NotFound().json(json!({
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
        Ok(updated_device) => HttpResponse::Ok().json(json!({
            "status": "success",
            "device": updated_device,
        })),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

#[delete("/devices/{id}")]
async fn delete_device_by_id(
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
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("{:?}", error)
        })),
    }
}

pub fn config(conf: &mut ServiceConfig) {
    let scope = scope("/api")
        .service(health_checker)
        .service(create_client)
        .service(get_all_clients)
        .service(get_client_by_id)
        .service(update_client_by_id)
        .service(delete_client_by_id)
        .service(create_device)
        .service(get_all_devices)
        .service(get_device_by_id)
        .service(update_device_by_id)
        .service(delete_device_by_id);

    conf.service(scope);
}
