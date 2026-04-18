mod tests;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use tokio::runtime::Runtime;

use tests::*;

// generates main function to run benches group
criterion_main!(benches);

// creates a group named "benches", with a number of functions
criterion_group! {
    name = benches;
    config = config();
    targets = channel_bench, gpsc_bench
}

fn config() -> Criterion {
    Criterion::default().sample_size(10)
}

pub fn channel_bench(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("worker count");
    let worker = [10, 100, 1000, 2000, 4000];

    for n_worker in worker {
        group.bench_with_input(
            BenchmarkId::new("tokio mpsc", n_worker),
            &n_worker,
            |b, &n_worker| b.to_async(&rt).iter(|| mspc(n_worker)),
        );

        group.bench_with_input(
            BenchmarkId::new("gpsc", n_worker),
            &n_worker,
            |b, &n_worker| b.to_async(&rt).iter(|| gpsc(n_worker)),
        );
    }
}

pub fn gpsc_bench(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("quick");
    let worker = [1000];

    for n_worker in worker {
        group.bench_with_input(
            BenchmarkId::new("solo_gpsc", n_worker),
            &n_worker,
            |b, &n_worker| b.to_async(&rt).iter(|| gpsc(n_worker)),
        );
    }
}
