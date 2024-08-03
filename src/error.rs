use std::fmt::{self, Debug};
use std::{error, io};
use std::result;

#[derive(Debug)]
pub enum KvStoreError {
    Io(io::Error),
}

impl fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            KvStoreError::Io(ref err) => write!(f, "IO error: {}", err),
        }
    }
}

pub type Result<T> = result::Result<T, KvStoreError>;
