use std::sync::atomic::{AtomicBool, AtomicU32};

use parking_lot::Mutex;
use tokio::sync::{Notify, Semaphore};

use super::container::GpscContainer;

#[derive(Debug)]
pub(crate) struct GpscQueue<C> {
    data: Mutex<C>,
    cap: usize,

    closed: AtomicBool,
    slots: Semaphore,

    n_sender: AtomicU32,
    tx_closed: Notify,

    rx_closed: Notify,
    rx_data_available: Notify,
    rx_full: Notify,
}

impl<C> GpscQueue<C>
where
    C: GpscContainer + Send + 'static,
{
    pub(crate) fn new(cap: usize) -> Self {
        GpscQueue {
            data: Mutex::new(C::new(cap)),
            cap,
            closed: false.into(),
            n_sender: 1.into(), // we always start with one sender
            tx_closed: Notify::new(),

            slots: Semaphore::new(cap),
            rx_closed: Notify::new(),
            rx_data_available: Notify::new(),
            rx_full: Notify::new(),
        }
    }

    pub(crate) async fn put(&self, msg: C::Message) -> Option<()> {
        // calling .enable() ensures that permits arent lost and the task gets woken
        // in case the channel gets closed in the meantime
        let f = self.tx_closed.notified();
        tokio::pin!(f);
        f.as_mut().enable(); // if this were to consume the permit, is_closed() would return false

        if self.is_closed() {
            return None;
        }

        let permit = tokio::select! {
            r = self.slots.acquire() => {
                r.expect("the semaphore never closes")
            }
            _ = f => {
                return None
            }
        };

        let mut guard = self.data.lock();
        guard.insert(msg);

        // we let the consumer add back permits
        permit.forget();

        if guard.len() == self.cap {
            self.rx_full.notify_one();
        }
        self.rx_data_available.notify_one();
        drop(guard);

        Some(())
    }

    pub(crate) async fn take(&self, buf: &mut C) -> Option<usize> {
        if self.is_closed() {
            return None;
        };

        tokio::select! {
            _ = self.rx_data_available.notified() => {}
            _ = self.rx_closed.notified() => {
                // we drain the remaining data?
                return None;
            }
        };

        let mut guard = self.data.lock();
        let n = guard.len();
        std::mem::swap(&mut *guard, buf);

        debug_assert_eq!(n + self.slots.available_permits(), self.cap);

        self.slots.add_permits(n);
        Some(n)
    }

    pub(crate) async fn take_max(&self, buf: &mut C) -> Option<usize> {
        if self.is_closed() {
            return None;
        };

        tokio::select! {
            _ = self.rx_full.notified() => {}
            _ = self.rx_closed.notified() => {
                // we drain the remaining data?
                return None;
            }
        };

        let mut guard = self.data.lock();
        let n = guard.len();
        std::mem::swap(&mut *guard, buf);

        debug_assert_eq!(n, self.cap);

        self.slots.add_permits(n);
        Some(n)
    }

    pub(crate) fn has_data(&self) -> bool {
        self.slots.available_permits() < self.cap
    }

    pub(crate) fn close(&self) {
        // store HAS to come before sending notifications
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
        self.rx_closed.notify_one();
        self.tx_closed.notify_one();
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn inc_sender(&self) {
        self.n_sender
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub(crate) fn decr_sender(&self) {
        let n = self
            .n_sender
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        if n == 1 {
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
        assert_eq!(rx.take(&mut rcv_buf).await.unwrap(), 100);
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

        assert_eq!(rx.take_max(&mut rcv_buf).await.unwrap(), 100);
    }

    #[tokio::test]
    async fn drop_tx() {
        let (tx, rx) = channel::<Vec<String>>(100);

        let mut rcv_buf = Vec::with_capacity(100);

        drop(tx);

        assert!(rx.take(&mut rcv_buf).await.is_err());
    }

    #[tokio::test]
    async fn drop_rx() {
        let (tx, rx) = channel::<Vec<String>>(100);

        drop(rx);
        assert!(tx.send("Hello".to_string()).await.is_err())
    }
}
