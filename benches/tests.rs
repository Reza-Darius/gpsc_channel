use std::{sync::Arc, time::Duration};

use rand::prelude::*;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinSet,
};

pub fn mspc_worker(tx: Sender<String>, n_worker: usize, delay: u64) -> JoinSet<()> {
    let tx = Arc::new(tx);
    let mut join_set = JoinSet::new();

    for _ in 0..n_worker {
        let tx = tx.clone();
        join_set.spawn(async move {
            loop {
                let rng = rand::rng().random_range(..delay);

                tokio::time::sleep(Duration::from_millis(rng)).await;
                if tx.send(String::from("Message")).await.is_err() {
                    return;
                };
            }
        });
    }
    join_set
}
pub async fn mspc_consumer(mut rx: Receiver<String>, n_msgs: usize) {
    let mut count = 0;

    while let Some(_) = rx.recv().await
        && count < n_msgs
    {
        count += 1;
    }
}

pub async fn mspc_consumer_limit(mut rx: Receiver<String>, n_msgs: usize, limit: usize) {
    let mut count = 0;
    let mut buf = Vec::with_capacity(limit);

    while count < n_msgs {
        count += rx.recv_many(&mut buf, limit).await;
        for _ in buf.iter() {
            count += 1;
        }
        buf.clear();
    }
}

pub fn gpsc_worker(
    tx: gpsc_channel::Sender<Vec<String>>,
    n_worker: usize,
    delay: u64,
) -> JoinSet<()> {
    let tx = Arc::new(tx);
    let mut join_set = JoinSet::new();

    for _ in 0..n_worker {
        let tx = tx.clone();
        join_set.spawn(async move {
            loop {
                let rng = rand::rng().random_range(..delay);

                tokio::time::sleep(Duration::from_millis(rng)).await;
                if tx.send(String::from("Message")).await.is_err() {
                    return;
                };
            }
        });
    }
    join_set
}

pub async fn gpsc_consumer(rx: gpsc_channel::Receiver<Vec<String>>, n_msgs: usize, cap: usize) {
    let mut count = 0;
    let mut buf = Vec::with_capacity(cap);

    while count < n_msgs {
        let _ = rx.take(&mut buf).await;
        for _ in buf.iter() {
            count += 1;
        }
    }
}
