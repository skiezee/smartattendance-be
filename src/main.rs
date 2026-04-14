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
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Starting Smart Attendance Backend...");

    // Connect to SurrealDB
    log::info!("Connecting to SurrealDB on ws://127.0.0.1:8000...");
    let db = Surreal::new::<Ws>("127.0.0.1:8000")
        .await
        .expect("Failed to connect to SurrealDB");

    // Sign in using the root credentials provided
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .expect("Failed to sign in to SurrealDB");

    // Select the namespace and database
    db.use_ns("smartattendance")
        .use_db("dev2")
        .await
        .expect("Failed to select namespace and database");

    log::info!("Successfully connected to SurrealDB (ns: smartattendance, db: dev2)");

    // Wrap the database client in web::Data to share it across routes
    let app_state = web::Data::new(AppState { db });

    // Start Actix Web Server on port 8080 (since SurrealDB uses 8000)
    let server_addr = "0.0.0.0:8080";
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
