use surrealdb::Surreal;
use surrealdb::engine::any::Any;

pub struct AppState {
    pub db: Surreal<Any>,
}
