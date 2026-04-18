mod tests;

use criterion::{Criterion, criterion_group, criterion_main};
use tests::*;

use tokio::runtime::Runtime;

// generates main function to run benches group
criterion_main!(benches);

// creates a group named "benches", with a number of functions
criterion_group! {
    name = benches;
    config = config();
    targets = channel_bench
}

fn config() -> Criterion {
    Criterion::default().sample_size(10)
}

// channel capacity
const BUFFER: usize = 1024;
// number of messages for a consumer to process
const N_MSGS: usize = 1000;
// number of simultaneous producer per consumer
const N_WORKER: usize = 100;
// random amount of time a worker waits before sending a msg in miliseconds
const DELAY: u64 = 10;

async fn mspc(n_msgs: usize, n_worker: usize, buffer: usize, delay: u64) {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(buffer);

    let mut handles = mspc_worker(tx, n_worker, delay);
    mspc_consumer(rx, n_msgs).await;

    handles.shutdown().await;
}

async fn mspc_limit(n_msgs: usize, n_worker: usize, buffer: usize, delay: u64) {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(buffer);

    let mut handles = mspc_worker(tx, n_worker, delay);
    mspc_consumer_limit(rx, n_msgs, buffer).await;

    handles.shutdown().await;
}

async fn gpsc(n_msgs: usize, n_worker: usize, buffer: usize, delay: u64) {
    let (tx, rx) = gpsc_channel::channel::<Vec<String>>(buffer);

    let mut handles = gpsc_worker(tx, n_worker, delay);
    gpsc_consumer(rx, n_msgs, buffer).await;

    handles.shutdown().await;
}

pub fn channel_bench(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("tokio mpsc recv", |b| {
        b.to_async(&rt)
            .iter(|| mspc(N_MSGS, N_WORKER, BUFFER, DELAY))
    });

    c.bench_function("tokio mpsc recv_many", |b| {
        b.to_async(&rt)
            .iter(|| mspc_limit(N_MSGS, N_WORKER, BUFFER, DELAY))
    });

    c.bench_function("gpsc", |b| {
        b.to_async(&rt)
            .iter(|| gpsc(N_MSGS, N_WORKER, BUFFER, DELAY))
    });
}
