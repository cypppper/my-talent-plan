use std::{
    fs::{self, File, OpenOptions}, 
    io::{BufWriter, Read, Write}, 
    os::unix::fs::FileExt, 
    path::PathBuf, 
    sync::Arc
};
use crate::error::Result;

pub const WAL_FILE_NAME: &str = "wal.log";
pub const WAL_FILE_DIR: &str = "wal";

fn get_wal_file_name(id: Option<usize>) -> PathBuf {
    if let Some(id) = id {
        PathBuf::from(format!("imm-wal{:05}.log", id))
    } else {
        PathBuf::from(WAL_FILE_NAME)
    }
}

#[derive(Clone)]
pub struct WAL {
    wal_file: Arc<File>,
    dir_path: PathBuf,
    file_sz: usize,

    next_imm_wal_idx: usize,
}

impl WAL {
    /// dir_path: work dir
    fn check_and_create_dir(dir_path: &PathBuf) {
        let dir_path = dir_path.join(WAL_FILE_DIR);
        if !dir_path.is_dir() {
            fs::create_dir(dir_path).unwrap();
        }
    }

    /// dir_path: work dir path
    /// ok for both reopen and new_open
    /// read next_imm_wal_idx
    pub fn open(dir_path: PathBuf) -> Result<Self> {
        Self::check_and_create_dir(&dir_path);
        let dir_path = dir_path.join(WAL_FILE_DIR);
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open((&dir_path).join(WAL_FILE_NAME))?;
        let file_sz = file.metadata().unwrap().len() as usize;
        
        let dir = fs::read_dir(&dir_path).unwrap();
        let next_imm_wal_idx = dir.count() - 1;
        Ok(Self {
            wal_file: Arc::new(file),
            dir_path,
            file_sz,
            next_imm_wal_idx,
        })
    }

    pub fn get_imm_num(&self) -> usize {
        self.next_imm_wal_idx
    }

    /// every write is a 4byte log-len + k byte json log
    pub fn write(&mut self, buffer: &[u8]) -> Result<()> {
        let mut writter = BufWriter::new(self.wal_file.clone());
        writter.write_all(buffer)?;
        writter.flush()?;
        self.file_sz += buffer.len();
        Ok(())
    }

    pub fn read_one_file(&self, id: Option<usize>) -> Result<Vec<u8>> {
        let wal_name = get_wal_file_name(id);
        let mut wal_content: Vec<u8> = Vec::new();
        let mut wal_file = OpenOptions::new()
            .read(true)
            .create(false)
            .open((&self.dir_path).join(wal_name)).unwrap();
        let wal_size = wal_file.metadata().unwrap().len() as usize;
        wal_content.resize(wal_size, 0);

        assert_eq!(wal_file.read(&mut wal_content)?, wal_size);
        Ok(wal_content)
    }
    /// id == null: wal_file
    /// id != null: imm_wal_file (0-based)
    pub fn read_offset(&self, id: Option<usize>, offset: usize, len: usize) -> Result<Vec<u8>> {
        let wal_name = get_wal_file_name(id);
        let mut ret: Vec<u8> = Vec::new();
        ret.resize(len, 0);

        let wal_file = OpenOptions::new()
            .read(true)
            .create(false)
            .open((&self.dir_path).join(wal_name))?;
        wal_file.read_exact_at(&mut ret, offset as u64).unwrap();
        Ok(ret)
    }
    pub fn get_wal_sz(&self) -> usize {
        self.file_sz
    }
    pub fn get_freeze_file(&self) -> BufWriter<File> {
        let file_name = get_wal_file_name(Some(self.next_imm_wal_idx));
        let ret = OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.dir_path.join(file_name))
            .unwrap();
        BufWriter::new(ret)
    }
    pub fn incr_next_wal(&mut self) {
        self.file_sz = 0;
        self.next_imm_wal_idx += 1;
        self.wal_file.set_len(0).unwrap();
        self.wal_file.flush().unwrap();
    }
}
