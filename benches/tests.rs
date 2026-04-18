use tokio::{io::AsyncWriteExt, task::JoinSet};

// channel capacity
const BUFFER: usize = 2000;
// number of messages for a consumer to process
const N_MSGS: usize = 1000;

pub async fn mspc(n_worker: usize) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(BUFFER);

    let consumer = tokio::spawn(async move {
        let mut count = 0;
        let mut sink = tokio::io::sink();

        while let Some(msg) = rx.recv().await
            && count < N_MSGS
        {
            let _ = sink.write(msg.as_bytes()).await.unwrap();
            count += 1;
        }
    });

    let mut worker = JoinSet::new();

    for _ in 0..n_worker {
        let tx = tx.clone();
        worker.spawn(async move {
            loop {
                if tx.send(String::from("Message")).await.is_err() {
                    return;
                };
            }
        });
    }

    let _ = consumer.await;
    worker.shutdown().await;
}

pub async fn gpsc(n_worker: usize) {
    let (tx, rx) = gpsc_channel::channel::<Vec<String>>(BUFFER);

    let consumer = tokio::spawn(async move {
        let mut count = 0;
        let mut buf = Vec::with_capacity(BUFFER);
        let mut sink = tokio::io::sink();

        loop {
            // let now = Instant::now();
            let _ = rx.swap(&mut buf).await.unwrap();
            for msg in buf.iter() {
                if count == N_MSGS {
                    return;
                }
                let _ = sink.write(msg.as_bytes()).await.unwrap();
                count += 1;
            }
            buf.clear();
        }
    });

    let mut worker = JoinSet::new();

    for _ in 0..n_worker {
        let tx = tx.clone();
        worker.spawn(async move {
            loop {
                if tx.send(String::from("Message")).await.is_err() {
                    return;
                };
            }
        });
    }

    let _ = consumer.await;
    worker.shutdown().await;
}
