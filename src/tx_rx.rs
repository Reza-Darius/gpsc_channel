use std::sync::Arc;

use crate::{container::BatchChanContainer, error::BatchChanError, queue::BatchQueue};

pub fn channel<C>(cap: usize) -> (BatchSender<C>, BatchReceiver<C>)
where
    C: BatchChanContainer + Send + 'static,
{
    if cap > usize::MAX >> 3 {
        panic!("invalid capacity")
    }

    let q = Arc::new(BatchQueue::new(cap));

    (
        BatchSender { inner: q.clone() },
        BatchReceiver { inner: q.clone() },
    )
}

#[derive(Debug)]
pub struct BatchReceiver<C>
where
    C: BatchChanContainer + Send + 'static,
{
    pub(crate) inner: Arc<BatchQueue<C>>,
}

impl<C> BatchReceiver<C>
where
    C: BatchChanContainer + Send + 'static,
{
    /// exchanges queue data with buffer, this call waits until data is available
    ///
    /// errors if the passed buffer isnt empty
    ///
    /// returns the amounts of messages retrieved
    pub async fn take(&self, buf: &mut C) -> Result<usize, BatchChanError> {
        if buf.len() != 0 {
            return Err(BatchChanError::Take(
                "exchange container is not empty".to_string(),
            ));
        }

        Ok(self.inner.take(buf).await)
    }

    /// exchanges queue data with buffer, remaining data in buf will be lost
    ///
    /// returns the amounts of messages retrieved
    pub async fn take_unchecked(&self, buf: &mut C) -> usize {
        buf.clear();
        self.inner.take(buf).await
    }

    /// exchanges queue data with buffer, this call waits until the channel is full
    ///
    /// errors if the passed buffer isnt empty
    ///
    /// returns the amounts of messages retrieved
    pub async fn take_max(&self, buf: &mut C) -> Result<usize, BatchChanError> {
        if buf.len() != 0 {
            return Err(BatchChanError::Take(
                "exchange container is not empty".to_string(),
            ));
        }

        Ok(self.inner.take_max(buf).await)
    }

    /// exchanges queue data with buffer, this call waits until the channel is full
    ///
    /// remaining data in buf will be lost
    ///
    /// returns the amounts of messages retrieved
    pub async fn take_max_unchecked(&self, buf: &mut C) -> usize {
        buf.clear();
        self.inner.take_max(buf).await
    }

    /// checks if the channel has data to read from
    pub fn has_date(&self) -> bool {
        self.inner.has_data()
    }
}

impl<C> Drop for BatchReceiver<C>
where
    C: BatchChanContainer + Send + 'static,
{
    fn drop(&mut self) {
        self.inner.close();
    }
}

/// cheaply clonable handle
#[derive(Debug, Clone)]
pub struct BatchSender<C>
where
    C: BatchChanContainer + Send + 'static,
{
    pub(crate) inner: Arc<BatchQueue<C>>,
}

impl<C> BatchSender<C>
where
    C: BatchChanContainer + Send + 'static,
{
    /// sends a message, waits if the queue is full
    ///
    /// returns Err if the channel is closed
    pub async fn send(&self, msg: C::Message) -> Result<(), BatchChanError> {
        if self.inner.is_closed() {
            return Err(BatchChanError::ChannelClosed);
        }
        self.inner.put(msg).await;
        Ok(())
    }
}
