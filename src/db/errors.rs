use actix_web::{error};
use thiserror::Error;
use std::{io, path::PathBuf};

// Set of different databaseerrors with thiserror
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("newly created database folder is not empty")]
    DatabaseNotEmpty,

    #[error("IO error")]
    IOError(#[from] io::Error),

    #[error("\"worlds\" folder not found at path {0}")]
    WorldsFolderNotFound(PathBuf),

    #[error("player not found")]
    PlayerNotFound,

    #[error("serde serialization error")]
    SerializationFailed(#[from] serde_json::Error)
}

impl error::ResponseError for DatabaseError {}