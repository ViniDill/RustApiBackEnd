use actix_web::{
    web::{Json, Path, Data, Query},
    get, post, patch, delete, HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    schema::{CreateClientSchema, FilterOptions, UpdateClientSchema},
    model::ClientModel,
    AppState,
};
#[allow(unused_imports)]
use utoipa::ToSchema;

#[utoipa::path(
    request_body = CreateClientSchema,
    responses(
        (status = 200, description = "Create a new client.", body = ClientModel),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Clientes"
)]
#[post("/clients")]
pub async fn create_client(
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

#[utoipa::path(
    responses(
        (status = 200, description = "Get all clients.", body = [ClientModel]),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Clientes"
)]
#[get("/clients")]
pub async fn get_all_clients(
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

#[utoipa::path(
    responses(
        (status = 200, description = "Get client by ID.", body = ClientModel),
        (status = 404, description = "Client not found."),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Clientes"
)]
#[get("/clients/{id}")]
pub async fn get_client_by_id(
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

#[utoipa::path(
    request_body = UpdateClientSchema,
    responses(
        (status = 200, description = "Update client by ID.", body = ClientModel),
        (status = 404, description = "Client not found."),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Clientes"
)]
#[patch("/clients/{id}")]
pub async fn update_client_by_id(
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

#[utoipa::path(
    responses(
        (status = 204, description = "Delete client by ID."),
        (status = 404, description = "Client not found."),
        (status = 500, description = "Internal Server error.")
    ),
    tag = "Clientes"
)]
#[delete("/clients/{id}")]
pub async fn delete_client_by_id(
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
