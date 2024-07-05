use mini_redis::{client, Result};
use tokio::sync::mpsc;
use bytes::Bytes;
use tokio::sync::oneshot;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<Option<Result<T>>>;

#[tokio::main]
async fn main() {
    
    let (tx, mut rx) = mpsc::channel(32);


    let tx2 = tx.clone();


    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cd = Command::Set { key: "foo".to_string(), value: "good".into(), resp: resp_tx };

        tx.send(cd).await.unwrap();

        let res = resp_rx.await;
        println!("{:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };

        tx2.send(cd).await.unwrap();

        let res = resp_rx.await;
        println!("{:?}", res);
    });




    let manager = tokio::spawn(async move {
        let mut cli = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Set { key, value, resp } => {
                    let res = cli.set(&key, value).await;
                    let _ = resp.send(Some(res));
                }
                Command::Get { key, resp } => {
                    let res = cli.get(&key).await;

                    let _ = resp.send(Some(res));

                }
            }
        }

        let _ = cli.set("ab", "cd".into()).await;
        
    });



    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();

}