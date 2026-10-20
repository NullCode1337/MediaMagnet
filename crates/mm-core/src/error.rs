#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Msg(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("task failed: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("internal lock poisoned")]
    Lock,
    #[error(transparent)]
    Tool(#[from] crate::runtime::ToolError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Msg(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::Msg(value.to_string())
    }
}
