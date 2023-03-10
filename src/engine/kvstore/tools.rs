use crate::error::Result;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;
use crate::engine::kvstore::command::{Command, CommandPos};
use crate::engine::kvstore::io::{BufReaderWithOffset, BufWriterWithOffset};

pub struct FileNameGenerator {
    pub(crate) current: u64,
    pub(crate) extension: String,
}

impl FileNameGenerator {
    pub fn new(extension: &str) -> Self {
        Self {
            current: 0,
            extension: extension.to_string(),
        }
    }

    pub fn current(&self) -> String {
        self.current.to_string() + "." + &self.extension
    }

    pub fn flush(&mut self, current: u64) {
        if current > self.current {
            self.current = current
        }
    }

    pub fn next(&mut self) -> String {
        self.current += 1;
        self.current()
    }
}

/// Read commands from log file and store them to key_map
/// In consideration of data consistency, the `reader` must be a `BufReaderWithOffset`.
///
/// # Arguments
/// * `key_map` hash map that read command will be stored in
/// * `reader` buf reader with offset, read commands from it
pub fn read_log(
    file_stem: u64,
    key_map: &mut HashMap<String, CommandPos>,
    reader: &mut BufReaderWithOffset<File>,
) -> Result<u64> {
    let mut offset = reader.seek(SeekFrom::Start(0))?;
    let mut uncompacted = 0u64;
    let mut stream = serde_json::Deserializer::from_reader(reader).into_iter::<Command>();
    while let Some(cmd) = stream.next() {
        let current_offset = stream.byte_offset() as u64;
        let cmd_pos = CommandPos::new(file_stem, offset, current_offset - offset);
        match cmd? {
            Command::SetCommand { key, .. } => {
                if let Some(old_cmd) = key_map.insert(key, cmd_pos) {
                    uncompacted += old_cmd.len;
                }
            }
            Command::RemoveCommand { key } => {
                // if already contains this
                if let Some(old_cmd) = key_map.remove(&key) {
                    uncompacted += old_cmd.len;
                }
                uncompacted += current_offset - offset;
            }
        }
        offset = current_offset;
    }
    Ok(uncompacted)
}

/// Collect log file names in given directory
/// # Examples
/// ```rust
/// use std::fs::File;
/// use tempfile::TempDir;
/// use kvs::engine::kvstore::tools::collect_file_stems;
/// let tempdir = TempDir::new().expect("failed to create temporary directory");
/// File::create(tempdir.path().to_str().unwrap().to_owned() + "/0.log").expect("failed to create file 0.log");
/// File::create(tempdir.path().to_str().unwrap().to_owned() + "/1.log").expect("failed to create file 1.log");
/// assert_eq!(collect_file_stems(tempdir.path()).unwrap(), vec![0u64, 1u64]);
/// ```
pub fn collect_file_stems(path: impl Into<PathBuf>) -> Result<Vec<u64>> {
    let path = path.into();
    let mut file_stems: Vec<u64> = fs::read_dir(&path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension().eq(&Some("log".as_ref())))
        .flat_map(|path| {
            path.file_stem()
                .and_then(OsStr::to_str)
                // .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    file_stems.sort();
    Ok(file_stems)
}

/// Open file as buf_writer with given path and file_stem
pub fn new_writer(dir_path: &(impl Into<PathBuf> + Clone), file_stem: u64) -> Result<BufWriterWithOffset<File>> {
    let mut writer_path: PathBuf = (*dir_path).clone().into();
    writer_path.push(file_stem.to_string() + ".log");
    let new_log = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(writer_path)?;
    let writer = BufWriterWithOffset::new(new_log)?;
    Ok(writer)
}

/// Open file as buf_reader with given path and file_stem
pub fn new_reader(dir_path: &(impl Into<PathBuf> + Clone), file_stem: u64) -> Result<BufReaderWithOffset<File>> {
    let mut reader_path: PathBuf = (*dir_path).clone().into();
    reader_path.push(file_stem.to_string() + ".log");
    let new_log = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(reader_path)?;
    let reader = BufReaderWithOffset::new(new_log)?;
    Ok(reader)
}

#[cfg(test)]
mod filename_generator_tests {
    use super::FileNameGenerator;

    // should generate string "0.txt"
    #[test]
    fn generate_name_test() {
        let gen = FileNameGenerator::new("txt");
        assert_eq!(gen.current(), "0.txt");
    }

    // should generate string "2.txt"
    #[test]
    fn next_name_test() {
        let mut gen = FileNameGenerator::new("txt");
        gen.next();
        gen.next();
        assert_eq!(gen.current(), "2.txt");
    }

    // should not flush
    #[test]
    fn no_flush_name_test() {
        let mut gen = FileNameGenerator::new("txt");
        gen.next();
        gen.next();
        assert_eq!(gen.current(), "2.txt");
        gen.flush(1);
        assert_eq!(gen.current(), "2.txt");
    }

    // should flush
    #[test]
    fn flush_name_test() {
        let mut gen = FileNameGenerator::new("txt");
        gen.next();
        gen.next();
        assert_eq!(gen.current(), "2.txt");
        gen.flush(3);
        assert_eq!(gen.current(), "3.txt");
    }
}