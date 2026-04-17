use thiserror::Error;

#[derive(Debug, Error)]
pub enum GpscError {
    #[error("{0}")]
    Take(String),
    #[error("channel is closed")]
    ChannelClosed,
}
