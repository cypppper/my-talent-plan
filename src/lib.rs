mod error;

use std::path::PathBuf;
pub use error::Result;

pub struct KvStore {

}

impl KvStore {
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        todo!()
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        todo!()
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        todo!()
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        todo!()
    }

    pub fn new() -> Self {
        todo!()
    }
}
