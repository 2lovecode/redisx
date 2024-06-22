use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;


#[tokio::main]
async fn main() {
    let listener  = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    let mut conn = Connection::new(socket);
    use mini_redis::Command::{self, Get, Set};

    let mut db:HashMap<String, Vec<_>> = HashMap::new();

    while let Some(frame) = conn.read_frame().await.unwrap() {
        let res = match Command::from_frame(frame).unwrap() {
            Get(cmd) => {
                match db.get(cmd.key()) {
                    Some(value) => {
                        Frame::Bulk(value.clone().into())
                    },
                    None => {
                        Frame::Null
                    }
                }
            }
            Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            cmd => panic!("unimplemented {:?}", cmd)
        };

        conn.write_frame(&res).await.unwrap();
    }
}