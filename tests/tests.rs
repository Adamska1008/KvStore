#![feature(is_some_and)]

#[cfg(test)]
mod kvs_tests {
    use assert_cmd::Command;
    use kvs::error::KvErrorKind;
    use kvs::KvError::KeyNotFound;
    use kvs::{KvError, KvStore, Result};
    use predicates::ord::eq;
    use predicates::prelude::PredicateStrExt;
    use predicates::str::{contains, is_empty};
    use tempfile::TempDir;
    use walkdir::WalkDir;

    // `kvs` with no args should exit with a none-zero code
    #[test]
    fn cli_no_args() {
        Command::cargo_bin("kvs").unwrap().assert().failure();
    }

    // `kvs -V` should print the version
    #[test]
    fn cli_version() {
        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["-V"])
            .assert()
            .stdout(contains(env!("CARGO_PKG_VERSION")));

        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["--version"])
            .assert()
            .stdout(contains(env!("CARGO_PKG_VERSION")));
    }

    // `kvs get <KEY>` should print "Key not found" for a non-existent key and exit with non-zero.
    #[test]
    fn cli_get_non_existent_key() {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["get", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .failure()
            .stdout(eq("Key not found: key1").trim());
    }

    // `kvs rm <KEY>` should print "Key not found" for an empty database and exit with non-zero code.
    #[test]
    fn cli_rm_non_existent_key() {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["rm", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .failure()
            .stdout(eq("Key not found").trim());
    }

    // `kvs set <KEY> <VALUE>` should print nothing and exit with zero.
    #[test]
    fn cli_set() {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["set", "key1", "value1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(is_empty());
    }

    // test `kvs` function on permanent storage
    #[test]
    fn cli_get_stored() -> Result<()> {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let mut kvs = KvStore::open(temp_dir.path())?;
        kvs.set("key1", "value1")?;
        kvs.set("key2", "value2")?;
        drop(kvs);

        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["get", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(eq("value1").trim());

        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["get", "key2"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(eq("value2").trim());

        Ok(())
    }

    // `kvs rm <KEY>` should print nothing and exit with zero.
    // after remove, `kvs get `<KEY>` should print "Key not found <KEY>" and exit with non-zero
    #[test]
    fn cli_rm_stored() -> Result<()> {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");

        let mut store = KvStore::open(temp_dir.path())?;
        store.set("key1", "value1")?;
        drop(store);

        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["rm", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(is_empty());

        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["get", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .failure()
            .stdout(eq("Key not found: key1").trim());

        Ok(())
    }

    // `kvs get` without param or with multiple params should exit without zero
    #[test]
    fn cli_invalid_get() {
        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["get"])
            .assert()
            .failure();

        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["get", "extra", "field"])
            .assert()
            .failure();
    }

    // `kvs set` without two params should exit without zero
    #[test]
    fn cli_invalid_set() {
        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["set"])
            .assert()
            .failure();

        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["set", "missing_field"])
            .assert()
            .failure();

        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["set", "extra", "extra", "field"])
            .assert()
            .failure();
    }

    // `kvs rm` with no params or more than one params should exit without zero
    #[test]
    fn cli_invalid_rm() {
        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["rm"])
            .assert()
            .failure();

        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["rm", "extra", "field"])
            .assert()
            .failure();
    }

    // `kvs` with unknown subcommand should exit without zero
    #[test]
    fn cli_invalid_subcommand() {
        Command::cargo_bin("kvs")
            .unwrap()
            .args(&["unknown", "subcommand"])
            .assert()
            .failure();
    }

    // Should get previously stored value.
    #[test]
    fn get_stored_value() -> Result<()> {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let mut store = KvStore::open(temp_dir.path())?;

        store.set("key1", "value1")?;
        store.set("key2", "value2")?;

        assert_eq!(store.get("key1")?, Some("value1".to_owned()));
        assert_eq!(store.get("key2")?, Some("value2".to_owned()));

        // Open from disk again and check persistent data.
        drop(store);
        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.get("key1")?, Some("value1".to_string()));
        assert_eq!(store.get("key2")?, Some("value2".to_string()));

        Ok(())
    }

    // Should overwrite existent value.
    #[test]
    fn overwrite_value() -> Result<()> {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let mut store = KvStore::open(temp_dir.path())?;

        store.set("key1", "value1")?;
        assert_eq!(store.get("key1")?, Some("value1".to_owned()));
        store.set("key1", "value2")?;
        assert_eq!(store.get("key1")?, Some("value2".to_owned()));

        // Open from disk again and check persistent data.
        drop(store);
        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.get("key1")?, Some("value2".to_owned()));
        store.set("key1", "value3")?;
        assert_eq!(store.get("key1")?, Some("value3".to_owned()));

        Ok(())
    }

    // Should get `None` when getting a non-existent key.
    #[test]
    fn get_non_existent_value() -> Result<()> {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let mut store = KvStore::open(temp_dir.path())?;

        store.set("key1", "value1")?;
        assert_eq!(store.get("key2")?, None);

        // Open from disk again and check persistent data.
        drop(store);
        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.get("key2")?, None);

        Ok(())
    }

    // Should get `None` when remove non-existent key.
    #[test]
    fn remove_non_existent_key() -> Result<()> {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let mut store = KvStore::open(temp_dir.path())?;
        assert_eq!(store.remove("key1")?, None);
        Ok(())
    }

    // should get `None` when getting a removed key.
    #[test]
    fn remove_key() -> Result<()> {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let mut store = KvStore::open(temp_dir.path())?;
        store.set("key1", "value1")?;
        store.remove("key1")?;
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
            // Compaction triggered.

            drop(store);
            // reopen and check content.
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

#[cfg(test)]
mod filename_generator_tests {
    use kvs::tools::FileNameGenerator;
    use predicates::ord::ge;

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
