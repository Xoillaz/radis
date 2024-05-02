use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;

// Multiple different commands are multiplexed over a single channel.
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

// Manager task may take the result back to the producer.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    // Generate a channel support multi-producer and one consumer.
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    // Move rx to manager task.
    let manager = tokio::spawn(async move {
        let mut client = client::connecti("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            // Err indicates the consumer being no longer interested.
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    // Ignore errors.
                    let _ = resp.send(res);
                }
                Command::Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    // Ignore errors.
                    let _ = resp.send(res);
                }
            }
        }
    });

    // Spawn two task, one setting a value;
    // The other querying for a key setted.
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };

        // Send the GET request.
        if tx.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        }

        // Await the response.
        let res = resp_rx.await;
        println!("GOT (Get) = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".to_string(),
            resp: resp_tx,
        };

        // Send the GET request.
        if tx2.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        }

        // Await the response.
        let res = resp_rx.await;
        println!("GOT (Get) = {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
