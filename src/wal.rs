use std::{fs::{rename, File, OpenOptions}, io::{BufWriter, Read, Seek, Write}, os::unix::fs::FileExt, path::{Path, PathBuf}, str::FromStr};
use crate::Result;

pub const WAL_FILE_NAME: &str = "log.log";
pub const WAL_FILE_BAK_NAME: &str = "log.bak";

pub struct WAL {
    writter: BufWriter<File>,
    dir_path: PathBuf,
    file_sz: usize,
}

impl WAL {
    pub fn open(dir_path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open((&dir_path).join(WAL_FILE_NAME))?;
        let file_sz = file.metadata().unwrap().len() as usize;
        Ok(Self {
            writter: BufWriter::new(file),
            dir_path,
            file_sz,
        })
    }
    pub fn new() -> Result<Self> {
        let wal_path = std::env::current_dir().unwrap().join(WAL_FILE_NAME);
        Ok(Self {
            writter: BufWriter::new(OpenOptions::new()
                .append(true)
                .create(true)
                .truncate(true)
                .open(&wal_path)?),
            dir_path: std::env::current_dir().unwrap(),
            file_sz: 0,
        })
    }
    pub fn new_bak(dir_path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open((&dir_path).join(WAL_FILE_BAK_NAME)).unwrap();
        let file_sz = file.metadata().unwrap().len() as usize;
        assert_eq!(file_sz, 0);
        Ok(Self {
            writter: BufWriter::new(file),
            dir_path,
            file_sz,
        })
    }
    /// every write is a 4byte log-len + k byte json log
    pub fn write(&mut self, buffer: &[u8]) -> Result<()> {
        self.writter.write_all(buffer)?;
        self.writter.flush()?;
        self.file_sz += buffer.len();
        Ok(())
    }
    pub fn sync(&mut self) -> Result<()> {
        self.writter.flush()?;
        Ok(())
    }
    pub fn read_all(&self) -> Result<Vec<u8>> {
        let mut wal_content: Vec<u8> = Vec::new();
        let mut wal_file = OpenOptions::new()
            .read(true)
            .create(false)
            .open((&self.dir_path).join(WAL_FILE_NAME)).unwrap();
        let wal_size = wal_file.metadata().unwrap().len() as usize;
        wal_content.resize(wal_size, 0);

        assert_eq!(wal_file.read(&mut wal_content)?, wal_size);
        Ok(wal_content)
    }
    pub fn read_offset(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        let mut ret: Vec<u8> = Vec::new();
        ret.resize(len, 0);
        let wal_file = OpenOptions::new()
            .read(true)
            .create(false)
            .open((&self.dir_path).join(WAL_FILE_NAME))?;
        wal_file.read_exact_at(&mut ret, offset as u64).unwrap();
        Ok(ret)
    }
    pub fn get_wal_sz(&self) -> usize {
        self.file_sz
    }
    pub fn rename_bak(&self) {
        rename((&self.dir_path).join(WAL_FILE_BAK_NAME), (&self.dir_path).join(WAL_FILE_NAME)).unwrap();
    }

}
