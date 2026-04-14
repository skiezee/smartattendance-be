#[tokio::main]
async fn main() -> Result<(), surrealdb::Error> {
    use surrealdb::engine::remote::ws::Ws;
    use surrealdb::opt::auth::Root;
    use surrealdb::Surreal;

    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.use_ns("smartattendance").use_db("dev2").await?;

    let mut result = db.query("SELECT * FROM employee WHERE nik = '1003';").await?;
    let records: Vec<crate::models::employee::Employee> = result.take(0).unwrap_or_default();
    println!("Found: {:?}", records);

    Ok(())
}
