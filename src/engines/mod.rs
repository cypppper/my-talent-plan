mod kvs;
use crate::error::Result;


pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        todo!()
    }
    fn get(&mut self, key: String) -> Result<Option<String>> {
        todo!()
    }
    fn remove(&mut self, key: String) -> Result<()> {
        todo!()
    }
}

pub use kvs::KvStore;

