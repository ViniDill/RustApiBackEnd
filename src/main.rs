mod schema;
mod model;
mod services;

use actix_cors::Cors; // <== IMPORTANTE
use actix_web::{
    web,
    App,
    HttpServer,
    middleware::Logger,
    http::header, // <== PARA HEADERS DO CORS
};

use dotenv::dotenv;
use sqlx::{
    postgres::PgPoolOptions,
    Postgres,
    Pool,
};

pub struct AppState {
    pub db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server started successfully");

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must to be set");

    let pool = match PgPoolOptions::new().max_connections(10).connect(&database_url).await {
        Ok(pool) => {
            println!("Connection DB resolved");
            pool
        }
        Err(error) => {
            println!("Failed to connect to the database: {:?}", error);
            std::process::exit(1)
        }
    };

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173") // <== ORIGEM DO FRONT-END
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors) // <== ADICIONA O CORS
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .configure(services::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
