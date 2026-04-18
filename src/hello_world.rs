use tokio::task::JoinSet;

use crate::channel;

#[tokio::main]
#[allow(dead_code)]
async fn main() {
    let (tx, rx) = channel::<Vec<String>>(100);

    let mut joins = JoinSet::new();

    for _ in 0..100 {
        let tx_clone = tx.clone();

        joins.spawn(async move {
            let data = String::from("hello");
            let _ = tx_clone.send(data).await;
        });
    }

    joins.join_all().await;

    let mut rcv_buf = Vec::with_capacity(100);
    let n = rx.swap(&mut rcv_buf).await.unwrap();

    assert_eq!(n, 100)
}
