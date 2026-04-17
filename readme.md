# An channel for those who like abusing MPSC channels

This channel works similarly to tokio's MSPC channel with one trick up it's sleeve.

Instead of reading message one at a time, it can clear the entire channel in a single operation by flipping the pointer between two buffer, making it really efficient for situations where you have a lot of producer and a single consumer.

this library was inspired by this article: https://blog.digital-horror.com/blog/how-to-avoid-over-reliance-on-mpsc/

## Quick Start

```Rust
  let (tx, rx) = channel::<Vec<String>>(100);

  let mut task_handles = vec![];

  for _ in 0..100 {
      let tx_clone = tx.clone();
      
      task_handles.push(tokio::spawn(async move {
          let data = String::from("hello");
          let _ = tx_clone.send(data).await;
      }));
  }

  for handle in task_handles {
      let _ = handle.await;
  }

  let mut rcv_buf = Vec::with_capacity(100);
  rx.take_unchecked(&mut rcv_buf).await;

  assert_eq!(rcv_buf.len(), 100)
```

### Container

Unlike regular channels, this channel is generic over a container that holds the type of message you want to send.

This library exposes the `GpscContainer` trait which is implemented for all the standard library collections.

## Benchmarks

![Screenshot](benchmark_comparison.png)
