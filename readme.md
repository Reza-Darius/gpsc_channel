# A channel for *lots* of producer

GPSC (gazillion producer, single consumer) is channel similar to tokio's MSPC channel with a twist:

Instead of reading message one at a time, it can clear the entire channel in a single operation by flipping the pointer between two buffer, making it really efficient in situations where you need backpressure and have a large amount of producer and a single consumer.

this library was inspired by this [article](https://blog.digital-horror.com/blog/how-to-avoid-over-reliance-on-mpsc/)

## Hello World

```Rust
use gpsc_channel::channel;
use tokio::task::JoinSet;

#[tokio::main]
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
```

### Container

Unlike regular channels, this channel is generic over a container that holds the type of message you want to send.

This library exposes the `GpscContainer` trait which is implemented for all the standard library collections.

## Benchmarks
![Screenshot](benchmark_comparison.png)

The reason this channel can be faster is two fold:
- avoiding the overhead of repeated calls to `recv()`
- using more efficient data structures: tokio's channel uses a linked list for its underlying memory
