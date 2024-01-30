use std::{
    io,
    path::PathBuf,
    path::Path,
};
use crate::hashsum;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("walkdir: {0}")]
    WalkDir(#[from] walkdir::Error),
    #[error("stripprefix: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
    #[error("failed to parse config file\n{0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("io: {0}, file: {1}")]
    IO(/*#[from]*/ io::Error, String),

    #[error("cannot sync symlink '{}'", .0.display())]
    Symlink(PathBuf),

    #[error("invalid command line argument '{0}'")]
    InvalidArgument(String),

    #[error("missing option to argument '{0}'")]
    MissingOption(String),

    #[error("incorrect hash size (should be '{}')", hashsum::HASH_SIZE)]
    BadHashSize,

    #[error("invalid game '{0}'")]
    InvalidGame(String),

    #[error("'{0}' is not configured for sync")]
    NotSyncable(String),
}

pub trait IntoErrorContext<T> {
    fn with_context(self, context: impl AsRef<Path>) -> Result<T, Error>;
}

impl<T> IntoErrorContext<T> for Result<T, io::Error> {
    fn with_context(self, context: impl AsRef<Path>) -> Result<T, Error> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(Error::IO(err, context.as_ref().display().to_string()))
        }
    }
}

