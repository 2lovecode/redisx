use tokio::{
    io::{
        AsyncReadExt, 
        AsyncWriteExt,
    }, 
    net::TcpListener,sync::mpsc
};


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6372").await.unwrap();

    let (sd,mut  rc) = mpsc::channel(2);

    let mut cnt = 0;

    tokio::spawn(async move {
        
        loop {
            match rc.recv().await {
                Some(a) => {
                    cnt += a;
                    println!("cnt: {}", cnt);
                },
                None => {},
            }
        }
        
    });

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        let sds = sd.clone();
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            sds.send(1).await.unwrap();
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        return;
                    },
                    Ok(n) => {
                        socket.write_all(&mut buf[..n]).await.unwrap();
                        return;
                    },
                    Err(_) => {
                        return;
                    },
                };
            }
        });
    }

}