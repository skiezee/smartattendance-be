mod config;
mod handlers;
mod models;
mod routes;
mod view_models;
mod services;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use crate::config::app_state::AppState;
use crate::routes::api::init_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file
    dotenv::dotenv().ok();

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Starting Smart Attendance Backend...");

    // Get environment variables
    let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let db_pass = std::env::var("DB_PASS").unwrap_or_else(|_| "root".to_string());
    let db_ns = std::env::var("DB_NS").unwrap_or_else(|_| "smartattendance".to_string());
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "dev2".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_addr = format!("0.0.0.0:{}", port);

    // Connect to SurrealDB
    let connection_url = if db_host.contains("://") {
        db_host.clone()
    } else {
        format!("ws://{}", db_host)
    };

    log::info!("Connecting to SurrealDB on {}...", connection_url);
    let db = Surreal::new::<Ws>(connection_url)
        .await
        .expect("Failed to connect to SurrealDB");

    // Sign in using the credentials provided
    db.signin(Root {
        username: &db_user,
        password: &db_pass,
    })
    .await
    .expect("Failed to sign in to SurrealDB");

    // Select the namespace and database
    db.use_ns(db_ns)
        .use_db(db_name)
        .await
        .expect("Failed to select namespace and database");

    log::info!("Successfully connected to SurrealDB");

    // Wrap the database client in web::Data to share it across routes
    let app_state = web::Data::new(AppState { db });

    // Start Actix Web Server
    log::info!("Starting Actix Web server on http://{}", server_addr);

    HttpServer::new(move || {
        let cors = Cors::permissive(); // Setup basic CORS for development

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .configure(init_routes)
    })
    .bind(server_addr)?
    .run()
    .await
}
