use std::sync::atomic::AtomicBool;

use parking_lot::Mutex;
use tokio::sync::{Notify, Semaphore};

use super::container::GpscContainer;

#[derive(Debug)]
pub(crate) struct GpscQueue<C> {
    q: Mutex<C>,
    cap: usize,
    closed: AtomicBool,
    n_sender: Mutex<u32>,

    slots: Semaphore,
    not_empty: Notify,
    not_full: Notify,
}

impl<C> GpscQueue<C>
where
    C: GpscContainer + Send + 'static,
{
    pub(crate) fn new(cap: usize) -> Self {
        GpscQueue {
            q: Mutex::new(C::new(cap)),
            cap,
            closed: false.into(),
            n_sender: 1.into(), // we always start with one sender
            slots: Semaphore::new(cap),
            not_empty: Notify::new(),
            not_full: Notify::new(),
        }
    }

    pub(crate) async fn put(&self, msg: C::Message) {
        let permit = self
            .slots
            .acquire()
            .await
            .expect("the semaphore never closes");

        let mut guard = self.q.lock();
        guard.insert(msg);

        // we let the consumer add back permits
        permit.forget();

        if guard.len() == self.cap {
            self.not_full.notify_one();
        }
        self.not_empty.notify_one();
    }

    pub(crate) async fn take(&self, buf: &mut C) -> usize {
        self.not_empty.notified().await;

        let mut guard = self.q.lock();
        let n = guard.len();
        std::mem::swap(&mut *guard, buf);

        self.slots.add_permits(n);
        n
    }

    pub(crate) async fn take_max(&self, buf: &mut C) -> usize {
        self.not_full.notified().await;

        let mut guard = self.q.lock();
        let n = guard.len();
        std::mem::swap(&mut *guard, buf);

        debug_assert_eq!(n, self.cap);

        self.slots.add_permits(n);
        n
    }

    pub(crate) fn has_data(&self) -> bool {
        self.slots.available_permits() < self.cap
    }

    pub(crate) fn close(&self) {
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn decr_sender(&self) {
        let mut guard = self.n_sender.lock();
        *guard -= 1;

        if *guard == 0 {
            self.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tx_rx::channel;

    #[tokio::test]
    async fn batch_chan() {
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

        assert_eq!(tx.inner.slots.available_permits(), tx.inner.cap);
        assert_eq!(rcv_buf.len(), 100)
    }

    #[tokio::test]
    async fn batch_chan_err() {
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
        rcv_buf.push("i will be lost!".to_string());

        assert!(rx.has_date());
        assert!(rx.take(&mut rcv_buf).await.is_err());
    }

    #[tokio::test]
    async fn batch_chan_max() {
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

        assert!(rx.has_date());
        rx.take_max_unchecked(&mut rcv_buf).await;

        assert_eq!(tx.inner.slots.available_permits(), tx.inner.cap);
        assert_eq!(rcv_buf.len(), 100)
    }
}
