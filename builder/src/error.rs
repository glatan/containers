use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("{command} の実行に失敗しました")]
    CommandFailed { command: &'static str },
    #[error("{command} の実行に失敗しました (ログ: {log})")]
    CommandFailedWithLog { command: &'static str, log: PathBuf },
    #[error("{command} の起動に失敗しました: {source}")]
    CommandSpawn {
        command: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
