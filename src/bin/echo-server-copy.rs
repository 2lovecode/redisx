use tokio::net::TcpListener;
use tokio::io;


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6371").await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let (mut rd, mut wd) = socket.split();
            io::copy(&mut rd, &mut wd).await.unwrap();
        });
    }

}