use core::str;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::sync::{Arc, Mutex};
use std::{collections::BTreeMap, path::PathBuf};
use bytes::{Buf, BufMut};

use super::{KVCommand, ENGINE_TAG_FILE};
use crate::wal::WAL;
use crate::error::Result;
use super::KvsEngine;

const TAG: &str = "kvss";

// : Clone + Send + 'static
pub struct KvStoreInner {
    wal: WAL,
    map_offset: BTreeMap<String, (Option<usize>, usize)>,
    work_dir: PathBuf,
}

pub struct KvStore {
    inner: Arc<Mutex<KvStoreInner>>
}


impl Clone for KvStore {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

unsafe impl Send for KvStore{}


impl KvStore {
    pub fn open(dir_path: impl Into<PathBuf>) -> Result<Self> {
        KvStoreInner::open(dir_path).map(|inn|
            Self{ inner: Arc::new(Mutex::new(inn)) }
        )
    }

    pub fn new() -> Result<Self> {
        KvStoreInner::new().map(|inn|
            Self{ inner: Arc::new(Mutex::new(inn)) }
        )
    }


}

impl KvStoreInner {
    pub fn open(dir_path: impl Into<PathBuf>) -> Result<Self> {
        let dir_path: PathBuf = dir_path.into();
        let wal = WAL::open(dir_path.clone()).unwrap();
        let mut ret = Self {
            wal,
            map_offset: BTreeMap::new(),
            work_dir: dir_path,
        };
        ret.init_from_wal();
        ret.check_config();
        Ok(ret)
    }

    // fn set_config(&self) {
    //     let mut file = OpenOptions::new()
    //         .create(true)
    //         .append(true)
    //         .open(self.work_dir.join(ENGINE_TAG_FILE))
    //         .unwrap();
    //     assert!(file.metadata().unwrap().size() == 0, "file path: {}", std::env::current_dir().unwrap().join(ENGINE_TAG_FILE).to_str().unwrap());
    //     file.write_all(TAG.as_bytes()).unwrap();
    // }

    fn check_config(&self) {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(self.work_dir.join(ENGINE_TAG_FILE))
            .unwrap();
        if file.metadata().unwrap().size() == 0 {
            file.write_all(TAG.as_bytes()).unwrap();
            return;
        }
        let mut read_content = Vec::<u8>::new();
        read_content.resize(4, 0);
        let read_len = file.read(&mut read_content[..]).unwrap();
        if !(read_len == 4 && str::from_utf8(&read_content[..]).unwrap() == TAG) {
            error!("last used must be kvs");
            std::process::exit(-1);
        }
    }

    fn init_from_wal(&mut self) {
        let imm_num = self.wal.get_imm_num();
        let mut offset_map = BTreeMap::<String, (Option<usize>, usize)>::new();

        for idx in 0..=imm_num {
            let idx = if idx == imm_num {None} else {Some(idx)};
            let wal_content = self.wal.read_one_file(idx).unwrap();
            let mut buffer = &wal_content[..];
            let mut offset: usize = 0;
            
            // println!("offset: {}, len: {}", offset, wal_content.len());
            while offset < wal_content.len() {
                let cmd_len = buffer.get_u32() as usize;
                // println!("get cmd len{}", cmd_len);
                offset += 4;  // size of u32
                let deserialized_content = serde_json::from_slice::<KVCommand>(&buffer[..cmd_len]).unwrap();
                
                match &deserialized_content {
                    KVCommand::Set(key, _) => {
                        offset_map.insert(key.to_owned(), (idx, offset));
                    },
                    KVCommand::Remove(key) => {
                        offset_map.remove(key);
                    },
                }
    
                buffer.advance(cmd_len);
                offset += cmd_len;
            }
            assert_eq!(offset, wal_content.len());
        }
        self.map_offset = offset_map;
    }

    pub fn new() -> Result<Self> {
        Self::open(std::env::current_dir().unwrap())
    }

    fn freeze_wal(&mut self) {
        let write_fn = |writter: &mut BufWriter<File>, buffer: &[u8]| {
            writter.write_all(buffer).unwrap();
            writter.flush().unwrap();
        };
        let mut imm_wal = self.wal.freeze_file();

        let mut imm_file_size = 0_usize;
        let mut wal_file_offset = 0_usize;
        let imm_file_idx = self.wal.get_imm_num();
        // write new log into imm
        while wal_file_offset < self.wal.get_wal_sz() {
            let cmd_log_len_bytes = self.wal.read_offset(None, wal_file_offset, 4).unwrap();
            wal_file_offset += 4;
            let cmd_log_len = (&cmd_log_len_bytes[..]).get_u32() as usize;
            let cmd_log_bytes = self.wal.read_offset(None, wal_file_offset, cmd_log_len).unwrap();

            if let Some(key) = KVCommand::deserialized_get_key(&&cmd_log_bytes[..]) {
                if let Some((idx, offset)) = self.map_offset.get(&key) {
                    if *offset == wal_file_offset {
                        // change offset_map
                        assert!(idx.is_none(), "current file idx must be wal(None)");
                        self.map_offset.insert(key, (Some(imm_file_idx), imm_file_size + 4));

                        // write to imm_file
                        let mut imm_cmd_serialized: Vec<u8> = Vec::new();
                        imm_cmd_serialized.put_u32(cmd_log_len as u32);
                        imm_cmd_serialized.extend(cmd_log_bytes);
                        write_fn(&mut imm_wal, &imm_cmd_serialized[..]);
                        imm_file_size += 4 + cmd_log_len;
                    }
                }
            }
            wal_file_offset += cmd_log_len;
        }
        assert_eq!(wal_file_offset, self.wal.get_wal_sz());
   
        self.wal.incr_next_wal();
    }

    fn check_and_do_freeze(&mut self) {
        let cur_size = self.wal.get_wal_sz();
        if cur_size >= 1000_000 {
            self.freeze_wal();
        }
    }
}


impl KvsEngine for KvStore {
    fn set(&self, key: String, value: String) -> Result<()> {

        let mut guard = self.inner.lock().unwrap();

        // write to map
        let offset = guard.wal.get_wal_sz() + 4;
        guard.map_offset.insert(key.clone(), (None, offset));
        // write to wal
        let cmd = KVCommand::Set(key, value.clone());
        let mut serialized: Vec<u8> = Vec::new();
        let cmd_serialized = serde_json::to_vec(&cmd).unwrap();
        serialized.put_u32(cmd_serialized.len() as u32);
        serialized.extend(cmd_serialized);
        guard.wal.write(&serialized).unwrap();
        
        // check and do compact
        guard.check_and_do_freeze();

        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let guard = self.inner.lock().unwrap();

        if let Some((file_idx, offset)) = guard.map_offset.get(&key) {
            let set_cmd_bytes_start = *offset;
            let set_cmd_bytes_len_bytes = guard.wal.read_offset(*file_idx, set_cmd_bytes_start - 4, 4).unwrap();
            let set_cmd_bytes_len = (&set_cmd_bytes_len_bytes[..]).get_u32() as usize;
            let set_cmd_bytes = guard.wal.read_offset(*file_idx, set_cmd_bytes_start, set_cmd_bytes_len).unwrap();
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

    fn remove(&self, key: String) -> Result<()> {
        let mut guard = self.inner.lock().unwrap();

        if guard.map_offset.contains_key(&key) {
            // remove in map
            guard.map_offset.remove(&key);
            // write to wal
            let cmd = KVCommand::Remove(key);
            let mut serialized: Vec<u8> = Vec::new();
            let cmd_serialized = serde_json::to_vec(&cmd).unwrap();
            serialized.put_u32(cmd_serialized.len() as u32);
            serialized.extend(cmd_serialized);
            guard.wal.write(&serialized).unwrap();
            Ok(())
        } else {
            Err(format_err!("Key not found").into())
        }
    }
}

// impl Drop for KvStore {
//     fn drop(&mut self) {
//         self.wal.sync().unwrap();
//     }
// }
