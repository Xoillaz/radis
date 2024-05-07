use mini_redis::{client, Result};

// A macro to start a runtime contains asynchronous task scheduler.
#[tokio::main]
// async fn returns an anonymous type implements the Future trait.
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address using TCP.
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set (key, value): ("hello", "world").
    client.set("hello", "world".into()).await?;

    // Get value satisfies the given key.
    let result = client.get("hello").await?;

    println!("Capture result from server = {:?}", result);

    Ok(())
}
