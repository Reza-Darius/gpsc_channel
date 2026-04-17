use thiserror::Error;

#[derive(Debug, Error)]
pub enum BatchChanError {
    #[error("{0}")]
    Take(String),
    #[error("channel is closed")]
    ChannelClosed,
}
