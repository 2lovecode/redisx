use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};


#[tokio::main]
async fn main() {
    let listener  = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        process(socket).await;
    }
}

async fn process(socket: TcpStream) {
    let mut conn = Connection::new(socket);

    if let Some(frame) = conn.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);
        conn.write_frame(&Frame::Error("This is an Error".into())).await.unwrap();
    }

    
}