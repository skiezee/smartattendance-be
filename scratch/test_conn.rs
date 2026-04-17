use surrealdb::engine::remote::ws::Ws;
use surrealdb::Surreal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoints = vec![
        "wss://smartatt.up.railway.app",
        "smartatt.up.railway.app",
        "ws://smartatt.up.railway.app",
    ];

    for ep in endpoints {
        println!("Testing endpoint: {}", ep);
        let res = Surreal::new::<Ws>(ep).await;
        match res {
            Ok(_) => println!("Successfully connected to {}", ep),
            Err(e) => println!("Failed to connect to {}: {:?}", ep, e),
        }
    }
    Ok(())
}
