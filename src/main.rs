mod schema;
mod model;
mod services;

use actix_cors::Cors;
use actix_web::{
    http::header,
    middleware::Logger,
    web, App, HttpResponse, HttpServer,
};
use dotenv::dotenv;
use sqlx::{
    postgres::PgPoolOptions,
    Postgres,
    Pool,
};
use utoipa::{
    openapi::{
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
    Modify, OpenApi, ToSchema,
};
use utoipa_swagger_ui::SwaggerUi;
use serde::{Serialize, Deserialize};

pub struct AppState {
    pub db: Pool<Postgres>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct TokenClaims {
    id: i32,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::services::create_client,
        crate::services::get_all_clients,
        crate::services::get_client_by_id,
        crate::services::update_client_by_id,
        crate::services::delete_client_by_id,

        crate::services::create_device,
        crate::services::get_all_devices,
        crate::services::get_device_by_id,
        crate::services::update_device_by_id,
        crate::services::delete_device_by_id,

        crate::services::health_checker,
    ),
    components(schemas(TokenClaims)),
    tags(
        (name = "Clientes", description = "Rotas relacionadas aos clientes"),
        (name = "Dispositivos", description = "Rotas relacionadas aos dispositivos"),
        (name = "Health", description = "Rotas para verificação do status da API"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
            components.add_security_scheme(
                "basic_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Basic)
                        .build(),
                ),
            );
        }
    }
}

// Função para servir o JSON da especificação OpenAPI
async fn openapi_json() -> HttpResponse {
    let openapi = ApiDoc::openapi();
    HttpResponse::Ok().json(openapi)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server started successfully");

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .service(
                web::scope("/docs")
                    .route("/swagger-ui/openapi.json", web::get().to(openapi_json))
                    .service(
                        SwaggerUi::new("/swagger-ui/{_:.*}")
                            .url("/docs/swagger-ui/openapi.json", ApiDoc::openapi())
                    )
            )
            .configure(services::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
