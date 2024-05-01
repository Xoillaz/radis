use mini_redis::{client, Result};

// Allow using async in fn main.
#[tokio::main]
// async returns type with Future trait, which needs await to execute.
async fn main() -> Result<()> {
    // Build connection with server mini-redis using TCP.
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set (key, value): ("hello", "world").
    client.set("hello", "world".into()).await?;

    // Get value satisfies the given key.
    let result = client.get("hello").await?;

    println!("Capture result from server = {:?}", result);

    Ok(())
}
