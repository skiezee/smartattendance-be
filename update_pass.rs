#[tokio::main]
async fn main() -> Result<(), surrealdb::Error> {
    use surrealdb::engine::remote::ws::Ws;
    use surrealdb::opt::auth::Root;
    use surrealdb::Surreal;
    use bcrypt::{hash, DEFAULT_COST};

    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.use_ns("smartattendance").use_db("dev2").await?;

    // Create a valid bcrypt hash for '12345'
    let password_hash = hash("12345", DEFAULT_COST).unwrap();

    let sql = format!("UPDATE employee SET password_hash = '{}' WHERE nik = '1003';", password_hash);
    let mut result = db.query(sql).await?;
    println!("Updated: {:?}", result);

    Ok(())
}
