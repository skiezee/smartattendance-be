use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

pub struct AppState {
    pub db: Surreal<Client>,
}
