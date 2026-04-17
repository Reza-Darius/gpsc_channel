mod tests;

use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use tests::*;

use tokio::runtime::Runtime;

// creates a group named "benches", with a number of functions
criterion_group! {
    benches,
    channel_benchmark
}

// generates main function to run benches group
criterion_main!(benches);

async fn mspc(n_msgs: usize, n_worker: usize, buffer: usize, delay: u64) {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(buffer);
    let handles = mspc_worker(tx, n_worker, delay);
    mspc_consumer(rx, n_msgs).await;

    assert_eq!(handles.len(), n_worker);
    for handle in handles {
        let _ = handle.await;
    }
}

async fn mspc_limit(n_msgs: usize, n_worker: usize, buffer: usize, delay: u64) {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(buffer);
    let handles = mspc_worker(tx, n_worker, delay);
    mspc_consumer_limit(rx, n_msgs, buffer).await;

    assert_eq!(handles.len(), n_worker);
    for handle in handles {
        let _ = handle.await;
    }
}

async fn gpsc(n_msgs: usize, n_worker: usize, buffer: usize, delay: u64) {
    let (tx, rx) = gpsc_channel::channel::<Vec<String>>(buffer);
    let handles = gpsc_worker(tx, n_worker, delay);
    gpsc_consumer(rx, n_msgs, buffer).await;

    assert_eq!(handles.len(), n_worker);
    for handle in handles {
        let _ = handle.await;
    }
}

pub fn channel_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // channel capacity
    let buffer = 1024;
    // random amount of time a worker waits before sending a msg in miliseconds
    let delay = 10;
    // number of messages for a consumer to process
    let n_msgs = 10000;
    // number of simultaneous producer per consumer
    let n_worker = 100;

    let mut group = c.benchmark_group("high throughput");
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    group.bench_function("tokio mpsc recv", |b| {
        b.to_async(&rt)
            .iter(|| mspc(n_msgs, n_worker, buffer, delay))
    });

    group.bench_function("tokio mpsc recv_many", |b| {
        b.to_async(&rt)
            .iter(|| mspc_limit(n_msgs, n_worker, buffer, delay))
    });

    group.bench_function("gpsc", |b| {
        b.to_async(&rt)
            .iter(|| gpsc(n_msgs, n_worker, buffer, delay))
    });

    group.finish();

    // channel capacity
    let buffer = 1024;
    // random amount of time a worker waits before sending a msg in miliseconds
    let delay = 5;
    // number of messages for a consumer to process
    let n_msgs = 100;
    // number of simultaneous producer per consumer
    let n_worker = 5;

    let mut group = c.benchmark_group("low throughput");
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    group.bench_function("tokio mpsc recv", |b| {
        b.to_async(&rt)
            .iter(|| mspc(n_msgs, n_worker, buffer, delay))
    });

    group.bench_function("tokio mpsc recv_many", |b| {
        b.to_async(&rt)
            .iter(|| mspc_limit(n_msgs, n_worker, buffer, delay))
    });

    group.bench_function("gpsc", |b| {
        b.to_async(&rt)
            .iter(|| gpsc(n_msgs, n_worker, buffer, delay))
    });

    group.finish();
}
