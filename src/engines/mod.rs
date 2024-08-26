mod kvs;
mod sled;
use crate::error::Result;


pub const ENGINE_TAG_FILE: &str = "engine.tag";

pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<()>;
}

use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug)]
enum KVCommand {
    Set(String, String),
    Remove(String),
}

impl KVCommand {
    pub fn deserialized_get_key(bytes: &[u8]) -> Option<String> {
        let deserialized_content = serde_json::from_slice::<KVCommand>(bytes).unwrap();
                
        match &deserialized_content {
            KVCommand::Set(key, _) => {
                Some(key.to_owned())
            },
            KVCommand::Remove(_) => {
                None
            },
        }
    }
}

pub use kvs::KvStore;
pub use sled::SledStore;

