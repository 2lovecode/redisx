use bytes::Bytes;
use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

type Db = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

const SHARE_NUM: usize = 10;

#[tokio::main]
async fn main() {
    let listener  = TcpListener::bind("127.0.0.1:6379").await.unwrap();



    let mut db_vec = Vec::with_capacity(SHARE_NUM);

    for _ in 0..SHARE_NUM {
        db_vec.push(Mutex::new(HashMap::new()));
    }

    let db:Db = Arc::new(db_vec);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db : Db) {
    let mut conn = Connection::new(socket);
    use mini_redis::Command::{self, Get, Set};

    while let Some(frame) = conn.read_frame().await.unwrap() {
        let res = match Command::from_frame(frame).unwrap() {
            Get(cmd) => {
                match string_to_int_hash(cmd.key()) {
                    Ok(hash_value) => {
                        let shard = db[hash_value as usize % SHARE_NUM].lock().unwrap();
                        match shard.get(cmd.key()) {
                            Some(value) => {
                                println!("{:?}", value);
                                Frame::Bulk(value.clone().into())
                            },
                            None => {
                                Frame::Null
                            }
                        }
                    },
                    Err(_) => {
                        Frame::Null
                    },
                }
            }
            Set(cmd) => {
                match string_to_int_hash(cmd.key()) {
                    Ok(hash_value) => {
                        let mut shard = db[hash_value as usize % SHARE_NUM].lock().unwrap();
                        shard.insert(cmd.key().to_string(), cmd.value().clone());
                        println!("a");
                        Frame::Simple("OK".to_string())
                    },
                    Err(e) => {
                        println!("{:?}", e);
                        Frame::Simple("OK".to_string())
                    },
                }
            }
            cmd => panic!("unimplemented {:?}", cmd)
        };

        conn.write_frame(&res).await.unwrap();
    }
}


fn string_to_int_hash(s: &str) -> Result<u64, std::num::ParseIntError> {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    Ok(hasher.finish())
}