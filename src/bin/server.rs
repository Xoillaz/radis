use bytes::Bytes;
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

// To support HashMap being shared across many tasks and potentially many threads.
type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    // Using Arc allows the HashMap to be referenced concurrently from many tasks,
    // potentially running on many threads.
    //
    // In case contention getting high and the lock is held across calls to .await,
    // you should use asynchronous mutex provided by tokio: tokio::sync::Mutex.
    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // The item ignored includes messages like IP.
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();

        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        //
        // The tokio::spawn function returns a JoinHandle, which the caller
        // may use to interact with the spawned task.
        //
        // Task spawned must implement Send, allows tasks moved between
        // threads while they are suspended at an .await.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

// Instead of initialization in a HashMap way, using a handle(Arc)
// provided by HashMap.
async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response to the client.
        connection.write_frame(&response).await.unwrap();
    }
}
