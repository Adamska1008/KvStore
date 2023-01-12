use crate::command::CommandPos;
use crate::error::Result;
use crate::io::{BufReaderWithOffset, BufWriterWithOffset};
use crate::tools::{read_log, FileNameGenerator, collect_file_stems};
use crate::{Command, tools};
use crate::KvError::UnexpectedCmdType;
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

const DEFAULT_COMPACTION_THRESHOLD: u64 = 1000;

/// A k-v database core, use log-structured.
pub struct KvStore {
    key_map: HashMap<String, CommandPos>,
    reader_map: HashMap<u64, BufReaderWithOffset<File>>,
    writer: BufWriterWithOffset<File>,
    generator: FileNameGenerator,
    uncompacted: u64,
    threshold: u64,
    dir_path: PathBuf
}

impl KvStore {
    /// Set string key-value in KvStore.
    /// # Arguments
    /// * `key` key string
    /// * `value` value string
    /// # Errors
    /// * `KvError::IoError` fail due to I/O errors
    /// * `KvError::SerdeError` fail parsing to ron string
    /// # Examples
    /// ```rust
    /// use tempfile::TempDir;
    /// use kvs::store::KvStore;
    /// let temp_dir = TempDir::new().unwrap();
    /// let mut kvs = KvStore::open(temp_dir.path()).unwrap();
    /// assert!(kvs.set("name", "Adam").is_ok());
    /// ```
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let cmd = Command::set(key, value).as_json()?;
        let offset = self.writer.offset;
        let len = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        let cmd_pos = CommandPos::new(self.generator.current, offset, len as u64);
        if let Some(old_cmd_pos) = self.key_map.insert(key.to_string(), cmd_pos) {
            self.uncompacted += old_cmd_pos.len;
        }
        self.compact();
        Ok(())
    }

    /// Get the string value of a string key.
    /// # Arguments
    /// * `key` key string
    /// # Errors
    /// * `KvError::KeyNotFound` key string is not found.
    /// * `KvError::IoError` fail due to I/O errors
    /// # Examples
    /// ```
    /// use tempfile::TempDir;
    /// use kvs::store::KvStore;
    /// let temp_dir = TempDir::new().expect("");
    /// let mut kvs = KvStore::open(temp_dir.path()).unwrap();
    /// kvs.set("name", "adam").unwrap();
    /// assert_eq!(kvs.get("name").unwrap(), Some("adam".to_owned()));
    /// assert_eq!(kvs.get("gender").unwrap(), None);
    /// ```
    pub fn get(&mut self, key: &str) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.key_map.get(key) {
            let reader = self.reader_map.get_mut(&cmd_pos.file_stem).expect(&format!(
                "Unable to find log file: {}.log",
                cmd_pos.file_stem
            ));
            reader.seek(SeekFrom::Start(cmd_pos.offset))?;
            let taker = reader.take(cmd_pos.len);
            let command: Command = serde_json::from_reader(taker)?;
            if let Command::SetCommand { value, .. } = command {
                Ok(Some(value))
            } else {
                Err(UnexpectedCmdType(command.name()))
            }
        } else {
            Ok(None)
        }
    }

    /// Remove a key-value pair.
    /// # Arguments
    /// * `key` key string
    /// # Errors
    /// * `KvError::IoError` fail due to I/O errors
    /// * `KvError::KeyNotFound` fail due to key not found
    /// # Examples
    /// ```
    /// use tempfile::TempDir;
    /// use kvs::store::KvStore;
    /// let temp_dir = TempDir::new().expect("");
    /// let mut kvs = KvStore::open(temp_dir.path()).unwrap();
    /// assert_eq!(kvs.remove("name").unwrap(), None);
    /// kvs.set("name", "Adam").unwrap();
    /// assert_eq!(kvs.remove("name").unwrap(), Some(()));
    /// assert_eq!(kvs.remove("name").unwrap(), None);
    /// ```
    pub fn remove(&mut self, key: &str) -> Result<Option<()>> {
        if let Some(old_cmd_pos) = self.key_map.remove(key) {
            self.uncompacted += old_cmd_pos.len;
            let cmd = Command::rm(key).as_json()?;
            self.writer.flush()?;
            self.writer.write(cmd.as_bytes())?;
            self.uncompacted += cmd.len() as u64;
            self.compact();
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    /// Open the KvStore at a given path.
    /// Return the KvStore.
    /// # Errors
    /// * `KvError::IoError`fail due to I/O errors
    /// # Examples
    /// ```rust
    /// use tempfile::TempDir;
    /// use kvs::store::KvStore;
    /// let temp_dir = TempDir::new().expect("");
    /// let kvs = KvStore::open(temp_dir.path()).expect("");
    /// ```
    pub fn open(dir_path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut uncompacted = 0u64;
        let mut generator = FileNameGenerator::new("log");
        let mut key_map = HashMap::new();
        let mut reader_map = HashMap::new();

        // get file paths from directory, filter all without extension "log" and transfer to file stem
        let dir_path = dir_path.into();
        fs::create_dir_all(&dir_path)?;
        let mut file_stems = collect_file_stems(&dir_path)?;
        if let Some(max) = file_stems.last() {
            generator.flush(max + 1);
        }
        file_stems.push(generator.current);

        // read data from log file
        for file_stem in file_stems {
            let mut file_path = dir_path.clone();
            file_path.push(file_stem.to_string() + ".log");
            let file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(file_path)?;
            let mut reader = BufReaderWithOffset::new(file)?;
            uncompacted += read_log(file_stem, &mut key_map, &mut reader)?;
            reader_map.insert(file_stem, reader);
        }


        let writer = tools::new_writer(&dir_path, &mut generator)?;
        Ok(Self {
            key_map,
            reader_map,
            writer,
            generator,
            uncompacted,
            threshold: DEFAULT_COMPACTION_THRESHOLD,
            dir_path
        })
    }

    pub fn compact(&self) {
        if self.uncompacted < self.threshold {
            return;
        }
    }

    /// Generate correct writer for database.
    /// There's only one writer in database.
    fn new_writer(&mut self) -> Result<()> {
        let writer = tools::new_writer(&self.dir_path, &mut self.generator)?;
        self.writer = writer;
        Ok(())
    }
}
