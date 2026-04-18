use std::sync::Arc;

use crate::{container::GpscContainer, error::GpscError, gpsc::GpscChan};

/// creates a new batch channel for a given collection and message type
///
/// panics if capacity exceeds `usize::MAX >> 3`
///
/// the sender is cheaply clonable
pub fn channel<C>(cap: usize) -> (Sender<C>, Receiver<C>)
where
    C: GpscContainer + Send + 'static,
{
    if cap > usize::MAX >> 3 {
        panic!("invalid capacity")
    }

    let q = Arc::new(GpscChan::new(cap));

    (Sender { inner: q.clone() }, Receiver { inner: q.clone() })
}

#[derive(Debug)]
pub struct Receiver<C>
where
    C: GpscContainer + Send + 'static,
{
    pub(crate) inner: Arc<GpscChan<C>>,
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
    pub async fn swap(&self, buf: &mut C) -> Result<usize, GpscError> {
        if buf.len() != 0 {
            return Err(GpscError::Take(
                "exchange container is not empty".to_string(),
            ));
        }

        self.inner.take(buf).await.ok_or(GpscError::ChannelClosed)
    }

    /// checks if the channel has data to read from
    pub fn has_data(&self) -> bool {
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
#[derive(Debug)]
pub struct Sender<C>
where
    C: GpscContainer + Send + 'static,
{
    pub(crate) inner: Arc<GpscChan<C>>,
}

impl<C> Clone for Sender<C>
where
    C: GpscContainer + Send + 'static,
{
    fn clone(&self) -> Self {
        self.new_sender()
    }
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
        self.inner.put(msg).await.ok_or(GpscError::ChannelClosed)
    }

    fn new_sender(&self) -> Self {
        self.inner.inc_sender();
        Sender {
            inner: self.inner.clone(),
        }
    }
}
