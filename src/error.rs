use std::fmt::{self, Debug};
use std::io;
use std::result;


#[derive(Debug, Fail)]
pub enum KvStoreError {
    Io(io::Error),
    Serde(serde_json::Error),
    StringErr(failure::Error),
}

impl fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Io(ref err) => write!(f, "IO error: {}", err),
            Self::Serde(ref err) => write!(f, "serde_json error: {}", err),
            Self::StringErr(ref err) => write!(f, "get error: {}", err),
        }
    }
}

impl From<serde_json::Error> for KvStoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl From<io::Error> for KvStoreError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<failure::Error> for KvStoreError {
    fn from(value: failure::Error) -> Self {
        Self::StringErr(value)
    }
}

pub type Result<T> = result::Result<T, KvStoreError>;
