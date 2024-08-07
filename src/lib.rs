#[macro_use]
extern crate failure;

mod error;
mod wal;

use serde::{Serialize, Deserialize};
use std::{collections::BTreeMap, path::PathBuf};
use bytes::{Buf, BufMut};

use wal::WAL;
pub use error::Result;

#[derive(Serialize, Deserialize, Debug)]
enum KVCommand {
    Set(String, String),
    Remove(String),
}

pub struct KvStore {
    wal: WAL,
    map_offset: BTreeMap<String, usize>,
    work_dir: PathBuf,
}

impl KvStore {
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // write to map
        let offset = self.wal.get_wal_sz() + 4;
        self.map_offset.insert(key.clone(), offset);
        // write to wal
        let cmd = KVCommand::Set(key, value.clone());
        let mut serialized: Vec<u8> = Vec::new();
        let cmd_serialized = serde_json::to_vec(&cmd).unwrap();
        serialized.put_u32(cmd_serialized.len() as u32);
        serialized.extend(cmd_serialized);
        self.wal.write(&serialized).unwrap();
        
        // check and do compact
        self.check_and_do_compact();

        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(offset) = self.map_offset.get(&key) {
            let set_cmd_bytes_start = *offset;
            let set_cmd_bytes_len_bytes = self.wal.read_offset(set_cmd_bytes_start - 4, 4).unwrap();
            let set_cmd_bytes_len = (&set_cmd_bytes_len_bytes[..]).get_u32() as usize;
            let set_cmd_bytes = self.wal.read_offset(set_cmd_bytes_start, set_cmd_bytes_len).unwrap();
            let set_cmd = serde_json::from_slice::<KVCommand>(&set_cmd_bytes[..]).unwrap();
            match set_cmd {
                KVCommand::Set(_, value) => {
                    Ok(Some(value))
                },
                _ => {unreachable!()}
            }
        } else {
            Ok(None)
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.map_offset.contains_key(&key) {
            // remove in map
            self.map_offset.remove(&key);
            // write to wal
            let cmd = KVCommand::Remove(key);
            let mut serialized: Vec<u8> = Vec::new();
            let cmd_serialized = serde_json::to_vec(&cmd).unwrap();
            serialized.put_u32(cmd_serialized.len() as u32);
            serialized.extend(cmd_serialized);
            self.wal.write(&serialized).unwrap();
            Ok(())
        } else {
            Err(format_err!("Key not found").into())
        }

    }

    pub fn open(dir_path: impl Into<PathBuf>) -> Result<KvStore> {
        let dir_path: PathBuf = dir_path.into();
        let wal = WAL::open(dir_path.clone()).unwrap();
        let mut ret = Self {
            wal,
            map_offset: BTreeMap::new(),
            work_dir: dir_path,
        };
        ret.init_from_wal();
        Ok(ret)
    }

    fn init_from_wal(&mut self) {
        let wal_content = self.wal.read_all().unwrap();
        let mut buffer = &wal_content[..];
        let mut offset: usize = 0;
        let mut offset_map = BTreeMap::<String, usize>::new();
        // println!("offset: {}, len: {}", offset, wal_content.len());
        while offset < wal_content.len() {
            let cmd_len = buffer.get_u32() as usize;
            // println!("get cmd len{}", cmd_len);
            offset += 4;  // size of u32
            let deserialized_content = serde_json::from_slice::<KVCommand>(&buffer[..cmd_len]).unwrap();
            
            match &deserialized_content {
                KVCommand::Set(key, _) => {
                    offset_map.insert(key.to_owned(), offset);
                },
                KVCommand::Remove(key) => {
                    offset_map.remove(key);
                },
            }

            buffer.advance(cmd_len);
            offset += cmd_len;
        }
        assert_eq!(offset, wal_content.len());
        self.map_offset = offset_map;
    }

    pub fn new() -> Result<Self> {
        Self::open(std::env::current_dir().unwrap())
    }

    pub fn wal_size(&self) -> usize {
        self.wal.get_wal_sz()
    }

    pub fn compact_wal(&mut self) {
        let mut new_wal = WAL::new_bak(self.work_dir.clone()).unwrap();

        // write new log into bak
        let new_offset_map: BTreeMap<String, usize> = self.map_offset.iter()
            .map(|(key, offset)| {
                let cmd_log_len_bytes = self.wal.read_offset(*offset - 4, 4).unwrap();
                let cmd_log_len = (&cmd_log_len_bytes[..]).get_u32() as usize;
                let cmd_log_bytes = self.wal.read_offset(*offset, cmd_log_len).unwrap();
                let mut bak_cmd_serialized: Vec<u8> = Vec::new();
                let bak_new_offset = new_wal.get_wal_sz() + 4;
                bak_cmd_serialized.put_u32(cmd_log_len as u32);
                bak_cmd_serialized.extend(cmd_log_bytes);
                new_wal.write(&bak_cmd_serialized[..]).unwrap();
                (key.to_owned(), bak_new_offset)
            })
            .collect();
        self.wal = new_wal;
        self.map_offset = new_offset_map;
        self.wal.rename_bak();
    }

    fn check_and_do_compact(&mut self) {
        let cur_size = self.wal_size();
        if cur_size >= 1000_000 {
            self.compact_wal();
        }
    }

}
