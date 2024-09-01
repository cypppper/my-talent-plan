use std::os::unix::fs::MetadataExt;
use std::path::{PathBuf, Path};
use std::fs::{OpenOptions, File};
use std::io::{Write, Read};
use std::str;

use sled::Db;

use super::{ENGINE_TAG_FILE};
use crate::error::Result;
use super::KvsEngine;

const SLED_DIR_NAME: &str = "sled";
const TAG: &str = "sled";

pub struct SledStore {
    db: Db,
    work_dir: PathBuf,
}

impl Clone for SledStore {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            work_dir: self.work_dir.clone(),
        }
    }
}

unsafe impl Send for SledStore {}


impl SledStore {
    pub fn open(work_dir: impl AsRef<Path>) -> Result<Self> {
        let sled_dir = work_dir.as_ref().join(SLED_DIR_NAME);
        let db = sled::open(&sled_dir).unwrap();
        // println!("{:?}", db.get("key2".as_bytes()));
        let ret = Self {
            db,
            work_dir: work_dir.as_ref().to_path_buf(),
        };
        ret.check_config();
        Ok(ret)
    }

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
            error!("last used must be sled");
            std::process::exit(-1);
        }    
    }
}

impl KvsEngine for SledStore {
    fn get(&self, key: String) -> Result<Option<String>> {
        let value = self.db.get(key.as_bytes()).unwrap();
        if value.is_some() {
            let value = String::from_utf8(value.unwrap().to_vec()).unwrap();
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    fn remove(&self, key: String) -> Result<()> {
        let result = self.db.remove(key.as_bytes()).unwrap();
        self.db.flush().unwrap();
        if result.is_some() {
            Ok(())
        } else {
            Err(format_err!("Key not found").into())
        }
    }
    fn set(&self, key: String, value: String) -> Result<()> {
        self.db.insert(key.as_bytes(), value.as_bytes()).unwrap();
        self.db.flush().unwrap();
        Ok(())
    }
}
