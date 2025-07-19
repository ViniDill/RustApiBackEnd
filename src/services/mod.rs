use actix_web::{web::{scope, ServiceConfig}, get, HttpResponse, Responder};
use serde_json::json;

pub mod clients;
pub mod devices;

pub use clients::*;
pub use devices::*;

/// Health check endpoint
#[utoipa::path(
    responses(
        (status = 200, description = "Health check: API is up and running smoothly.")
    ),
    tag = "Health"  // <<< Aqui você adiciona a tag
)]
#[get("/healthchecker")]
pub async fn health_checker() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Health check: API is up and running smoothly."
    }))
}

/// Configures the API routes
pub fn config(conf: &mut ServiceConfig) {
    let scope = scope("/api")
        .service(health_checker)
        // clientes
        .service(create_client)
        .service(get_all_clients)
        .service(get_client_by_id)
        .service(update_client_by_id)
        .service(delete_client_by_id)
        // devices
        .service(create_device)
        .service(get_all_devices)
        .service(get_device_by_id)
        .service(update_device_by_id)
        .service(delete_device_by_id);

    conf.service(scope);
}
