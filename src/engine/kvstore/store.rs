use crate::error::Result;
use crate::error::KvError::UnexpectedCmdType;
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use crate::engine::KvsEngine;
use crate::engine::kvstore::command::{Command, CommandPos};
use crate::engine::kvstore::io::{BufReaderWithOffset, BufWriterWithOffset};
use crate::engine::kvstore::tools::{self, FileNameGenerator};

const DEFAULT_COMPACTION_THRESHOLD: u64 = 1000;

/// A k-v database core, use log-structured.
pub struct KvStore {
    key_map: HashMap<String, CommandPos>,
    reader_map: HashMap<u64, BufReaderWithOffset<File>>,
    writer: BufWriterWithOffset<File>,
    generator: FileNameGenerator,
    uncompacted: u64,
    threshold: u64,
    dir_path: PathBuf,
}

impl KvsEngine for KvStore {
    /// Set string key-value in KvStore.
    /// # Arguments
    /// * `key` key string
    /// * `value` value string
    /// # Errors
    /// * `KvError::IoError` fail due to I/O errors
    /// * `KvError::SerdeError` fail parsing to json string
    /// # Examples
    /// ```rust
    /// use tempfile::TempDir;
    /// use kvs::engine::KvsEngine;
    /// use kvs::KvStore;
    /// let temp_dir = TempDir::new().unwrap();
    /// let mut kvs = KvStore::open(temp_dir.path()).unwrap();
    /// assert!(kvs.set("name", "Adam").is_ok());
    /// ```
    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let cmd = Command::set(key, value).as_json()?;
        let offset = self.writer.offset;
        let len = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        let cmd_pos = CommandPos::new(self.generator.current, offset, len as u64);
        if let Some(old_cmd_pos) = self.key_map.insert(key.to_string(), cmd_pos) {
            self.uncompacted += old_cmd_pos.len;
        }
        self.compact()?;
        Ok(())
    }

    /// Get the string value of a string key.
    /// # Arguments
    /// * `key` key string
    /// # Errors
    /// * `KvError::KeyNotFound` key string is not found.
    /// * `KvError::IoError` fail due to I/O errors
    /// # Return value
    /// * `Ok(Some(value))`: Key exists and corresponding value is `value`.
    /// * `Ok(None)`: Key not exists.
    /// * `Err(Error)`: Some error occurs.
    /// # Examples
    /// ```
    /// use tempfile::TempDir;
    /// use kvs::engine::KvsEngine;
    /// use kvs::KvStore;
    /// let temp_dir = TempDir::new().expect("");
    /// let mut kvs = KvStore::open(temp_dir.path()).unwrap();
    /// kvs.set("name", "adam").unwrap();
    /// assert_eq!(kvs.get("name").unwrap(), Some("adam".to_owned()));
    /// assert_eq!(kvs.get("gender").unwrap(), None);
    /// ```
    fn get(&mut self, key: &str) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.key_map.get(key) {
            let reader = self.reader_map.get_mut(&cmd_pos.file_stem).expect(&format!(
                "log file: {}.log is not cached in memory",
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
    /// # Return value
    /// * `OK(Some())`: Key exists and being successfully removed.
    /// * `Ok(None)`: Key not exists.
    /// * `Err(Error)`: Some error occurs.
    /// # Examples
    /// ```
    /// use tempfile::TempDir;
    /// use kvs::engine::KvsEngine;
    /// use kvs::KvStore;
    /// let temp_dir = TempDir::new().expect("");
    /// let mut kvs = KvStore::open(temp_dir.path()).unwrap();
    /// assert_eq!(kvs.remove("name").unwrap(), None);
    /// kvs.set("name", "Adam").unwrap();
    /// assert_eq!(kvs.remove("name").unwrap(), Some(()));
    /// assert_eq!(kvs.remove("name").unwrap(), None);
    /// ```
    fn remove(&mut self, key: &str) -> Result<Option<()>> {
        if let Some(old_cmd_pos) = self.key_map.remove(key) {
            self.uncompacted += old_cmd_pos.len;
            let cmd = Command::rm(key).as_json()?;
            self.writer.write(cmd.as_bytes())?;
            self.writer.flush()?;
            self.uncompacted += cmd.len() as u64;
            self.compact()?;
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}

impl KvStore {
    /// Open the KvStore at a given path.
    /// Return the KvStore.
    /// # Errors
    /// * `KvError::IoError`fail due to I/O errors
    /// # Examples
    /// ```rust
    /// use tempfile::TempDir;
    /// use kvs::KvStore;
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
        let mut file_stems = tools::collect_file_stems(&dir_path)?;
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
            uncompacted += tools::read_log(file_stem, &mut key_map, &mut reader)?;
            reader_map.insert(file_stem, reader);
        }

        let writer = tools::new_writer(&dir_path, generator.current)?;
        Ok(Self {
            key_map,
            reader_map,
            writer,
            generator,
            uncompacted,
            threshold: DEFAULT_COMPACTION_THRESHOLD,
            dir_path,
        })
    }

    /// After calling `compact` method, a new writer and log file will substitute the old one.
    ///
    /// All old file will be compacted into a new one, the only reader in memory correspond to
    /// the writer.
    pub fn compact(&mut self) -> Result<()> {
        if self.uncompacted < self.threshold {
            return Ok(());
        }
        self.generator.next();
        self.new_writer()?;
        for (key, cmd_pos) in self.key_map.iter_mut() {
            let reader = self.reader_map.get_mut(&cmd_pos.file_stem).expect(&format!(
                "log file: {}.log is not cached in memory",
                cmd_pos.file_stem
            ));
            reader.seek(SeekFrom::Start(cmd_pos.offset))?;
            let taker = reader.take(cmd_pos.len);
            let command: Command = serde_json::from_reader(taker)?;
            if let Command::SetCommand { value, .. } = command {
                let new_cmd_json = Command::set(key, &value).as_json()?;
                cmd_pos.file_stem = self.generator.current;
                cmd_pos.offset = self.writer.offset;
                cmd_pos.len = self.writer.write(new_cmd_json.as_bytes())? as u64;
            } else {
                return Err(UnexpectedCmdType(command.name()));
            }
        }
        self.reader_map.clear();
        for (file_stem, _) in self.reader_map.iter() {
            let mut reader_path = self.dir_path.clone();
            reader_path.push(file_stem.to_string() + ".log");
            fs::remove_file(reader_path)?;
        }
        let reader = tools::new_reader(&self.dir_path, self.generator.current)?;
        self.reader_map.insert(self.generator.current, reader);
        self.uncompacted = 0;
        Ok(())
    }

    /// Set compact threshold, default 1000, unit: bit.
    /// # Example
    /// ```rust
    /// use tempfile::TempDir;
    /// use kvs::KvStore;
    /// let temp_dir = TempDir::new().unwrap();
    /// let mut store = KvStore::open(temp_dir.path()).unwrap();
    /// store.set_compact_threshold(500u64);
    /// ```
    pub fn set_compact_threshold(&mut self, threshold: u64) {
        self.threshold = threshold;
    }

    /// Warning: after calling this method, the old writer will be removed from memory,
    /// but the reader which reads the same log file as old writer stays in memory.
    ///
    /// This method won't automatically add writer file to reader_map, if you hope so, do
    /// it manually.
    fn new_writer(&mut self) -> Result<()> {
        let writer = tools::new_writer(&self.dir_path, self.generator.current)?;
        self.writer = writer;
        Ok(())
    }
}


#[cfg(test)]
mod store_tests {
    use tempfile::TempDir;
    use walkdir::WalkDir;
    use super::KvsEngine;
    use super::KvStore;
    use super::Result;

    // Should get previous stored value after drop store and reopen
    #[test]
    fn get_stored_value() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let mut store = KvStore::open(temp_dir.path())?;

        store.set("key1", "value1")?;
        drop(store);
        let mut store = KvStore::open(temp_dir.path())?;
        store.set("key2", "value2")?;
        drop(store);

        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.get("key1")?, Some("value1".to_owned()));
        assert_eq!(store.get("key2")?, Some("value2".to_owned()));

        Ok(())
    }

    // Should overwrite existent value in any case
    #[test]
    fn overwrite_value() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let mut store = KvStore::open(temp_dir.path())?;
        store.set("key1", "value1")?;
        assert_eq!(store.get("key1")?, Some("value1".to_owned()));
        store.set("key1", "value2")?;
        assert_eq!(store.get("key1")?, Some("value2".to_owned()));
        drop(store);

        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.get("key1")?, Some("value2".to_owned()));
        store.set("key1", "value3")?;
        assert_eq!(store.get("key1")?, Some("value3".to_owned()));
        Ok(())
    }

    // Should get None when key is not exist
    #[test]
    fn get_non_existent_key() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let mut store = KvStore::open(temp_dir.path())?;
        store.set("key1", "value1")?;
        assert_eq!(store.get("key2")?, None);
        drop(store);

        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.get("key2")?, None);
        Ok(())
    }

    // Should get None when remove non-existent key
    #[test]
    fn remove_non_existent_key() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.remove("key1")?, None);
        Ok(())
    }

    // Should get Some(()) when remove key and get None when get removed key
    #[test]
    fn remove_key() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let mut store = KvStore::open(temp_dir.path())?;
        store.set("key1", "value1")?;
        assert_eq!(store.get("key1")?, Some("value1".to_owned()));
        drop(store);
        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.remove("key1")?, Some(()));
        assert_eq!(store.get("key1")?, None);
        drop(store);
        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.get("key1")?, None);
        Ok(())
    }

    // Insert data until total size of the directory decreases.
    // Test data correctness after compaction.
    #[test]
    fn compaction() -> Result<()> {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let mut store = KvStore::open(temp_dir.path())?;

        let dir_size = || {
            let entries = WalkDir::new(temp_dir.path()).into_iter();
            let len: walkdir::Result<u64> = entries
                .map(|res| {
                    res.and_then(|entry| entry.metadata())
                        .map(|metadata| metadata.len())
                })
                .sum();
            len.expect("fail to get directory size")
        };

        let mut current_size = dir_size();
        for iter in 0..1000 {
            for key_id in 0..1000 {
                let key = format!("key{}", key_id);
                let value = format!("{}", iter);
                store.set(&key, &value)?;
            }

            let new_size = dir_size();
            if new_size > current_size {
                current_size = new_size;
                continue;
            }
            // Compaction triggered

            drop(store);
            // reopen and check content
            let mut store = KvStore::open(temp_dir.path())?;
            for key_id in 0..1000 {
                let key = format!("key{}", key_id);
                assert_eq!(store.get(&key)?, Some(format!("{}", iter)));
            }
            return Ok(());
        }

        panic!("No compaction detected");
    }
}