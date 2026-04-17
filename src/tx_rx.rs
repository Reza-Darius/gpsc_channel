use std::sync::Arc;

use crate::{container::GpscContainer, error::GpscError, queue::GpscQueue};

/// creates a new mpsc batch channel for a given collection and message type
///
/// the sender is cheaply clonable
pub fn channel<C>(cap: usize) -> (Sender<C>, Receiver<C>)
where
    C: GpscContainer + Send + 'static,
{
    if cap > usize::MAX >> 3 {
        panic!("invalid capacity")
    }

    let q = Arc::new(GpscQueue::new(cap));

    (Sender { inner: q.clone() }, Receiver { inner: q.clone() })
}

#[derive(Debug)]
pub struct Receiver<C>
where
    C: GpscContainer + Send + 'static,
{
    pub(crate) inner: Arc<GpscQueue<C>>,
}

impl<C> Receiver<C>
where
    C: GpscContainer + Send + 'static,
{
    /// exchanges queue data with buffer, this call waits until data is available
    ///
    /// errors if the passed buffer isnt empty, or if the channel is closed
    ///
    /// returns the amounts of messages retrieved
    ///
    /// # Cancel Safety
    ///
    /// This function is cancel safe.
    pub async fn take(&self, buf: &mut C) -> Result<usize, GpscError> {
        if buf.len() != 0 {
            return Err(GpscError::Take(
                "exchange container is not empty".to_string(),
            ));
        }
        if self.inner.is_closed() {
            return Err(GpscError::ChannelClosed);
        }

        Ok(self.inner.take(buf).await)
    }

    /// exchanges queue data with buffer, remaining data in buf will be lost
    ///
    /// remaining data in buf will be lost, unless the channel is closed, in which case
    /// this function always immediately returns 0
    ///
    /// returns the amounts of messages retrieved
    ///
    /// # Cancel Safety
    ///
    /// This function is cancel safe.
    pub async fn take_unchecked(&self, buf: &mut C) -> usize {
        if self.inner.is_closed() {
            return 0;
        };
        buf.clear();
        self.inner.take(buf).await
    }

    /// exchanges queue data with buffer, this call waits until the channel is full
    ///
    /// errors if the passed buffer isnt empty, or if the channel is closed
    ///
    /// returns the amounts of messages retrieved
    ///
    /// # Cancel Safety
    ///
    /// This function is cancel safe.
    pub async fn take_max(&self, buf: &mut C) -> Result<usize, GpscError> {
        if buf.len() != 0 {
            return Err(GpscError::Take(
                "exchange container is not empty".to_string(),
            ));
        }
        if self.inner.is_closed() {
            return Err(GpscError::ChannelClosed);
        }

        Ok(self.inner.take_max(buf).await)
    }

    /// exchanges queue data with buffer, this call waits until the channel is full
    ///
    /// remaining data in buf will be lost, unless the channel is closed, in which case
    /// this function always immediately returns 0
    ///
    /// returns the amounts of messages retrieved
    ///
    /// # Cancel Safety
    ///
    /// This function is cancel safe.
    pub async fn take_max_unchecked(&self, buf: &mut C) -> usize {
        if self.inner.is_closed() {
            return 0;
        };
        buf.clear();
        self.inner.take_max(buf).await
    }

    /// checks if the channel has data to read from
    pub fn has_date(&self) -> bool {
        self.inner.has_data()
    }
}

impl<C> Drop for Receiver<C>
where
    C: GpscContainer + Send + 'static,
{
    fn drop(&mut self) {
        self.inner.close();
    }
}

/// cheaply clonable handle
#[derive(Debug, Clone)]
pub struct Sender<C>
where
    C: GpscContainer + Send + 'static,
{
    pub(crate) inner: Arc<GpscQueue<C>>,
}

impl<C> Drop for Sender<C>
where
    C: GpscContainer + Send + 'static,
{
    fn drop(&mut self) {
        self.inner.decr_sender();
    }
}

impl<C> Sender<C>
where
    C: GpscContainer + Send + 'static,
{
    /// sends a message, waits for free capacity
    ///
    /// returns Err if the channel is closed
    ///
    /// # Cancel Safety
    ///
    /// this function is cancel safe, only the position in the queue will be potentially lost
    pub async fn send(&self, msg: C::Message) -> Result<(), GpscError> {
        if self.inner.is_closed() {
            return Err(GpscError::ChannelClosed);
        }
        self.inner.put(msg).await;
        Ok(())
    }
}
