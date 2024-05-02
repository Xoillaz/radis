use bytes::Bytes;
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    // In case multiple .await calls should use asynchronous lock provided by tokio::sync::Mutex.
    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // The item ignored includes messages like IP.
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();

        // Every connection is moved to a new task, then returns a JoinHandle.
        // The lifetime of a task must be static.
        // Task must implement trait Send, allows tasks moving among threads.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

// Instead of HashMap init, using a handle(Arc) provided by HashMap.
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

        connection.write_frame(&response).await.unwrap();
    }
}
