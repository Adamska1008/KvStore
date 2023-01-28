mod cli_tests {
    use std::fs::File;
    use std::{fs, string, thread, time};
    use std::io::Write;
    use std::process::exit;
    use std::sync::mpsc;
    use std::time::Duration;
    use assert_cmd::Command;
    use tempfile::{tempdir, tempfile};
    use kvs::Result;
    use predicates::str;

    // Should exit with nonzero
    #[test]
    fn cli_no_args() -> Result<()> {
        let temp_dir = tempdir()?;
        Command::cargo_bin("kvs-client")
            .unwrap()
            .current_dir(&temp_dir)
            .assert()
            .failure();
        Ok(())
    }

    #[test]
    fn cli_invalid_get() -> Result<()> {
        let temp_dir = tempdir()?;
        // No args
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["get"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
        // Extra args
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["get", "key1", "key2"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
        // Invalid port
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["get", "key", "-p", "114514"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
        // Unknown flag
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["get", "key", "--unknown-flag"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
        Ok(())
    }

    #[test]
    fn cli_invalid_set() -> Result<()>{
        let temp_dir = tempdir()?;
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["set"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["set", "missing_field"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["set", "key", "value", "extra_field"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["set", "key", "value", "--addr", "invalid-addr"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["set", "key", "--unknown-flag"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
        Ok(())
    }

    #[test]
    fn cli_invalid_rm() {
        let temp_dir = tempdir().unwrap();
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["rm"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["rm", "extra", "field"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["rm", "key", "--addr", "invalid-addr"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["rm", "key", "--unknown-flag"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }

    #[test]
    fn cli_invalid_subcommand() {
        let temp_dir = tempdir().unwrap();
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["unknown"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }

    // kvs-client -V should print version
    #[test]
    fn cli_version() {
        let temp_dir = tempdir().unwrap();
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["-V"])
            .current_dir(&temp_dir)
            .assert()
            .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn server_version() {
        let temp_dir = tempdir().unwrap();
        Command::cargo_bin("kvs-server")
            .unwrap()
            .arg("-V")
            .current_dir(&temp_dir)
            .assert()
            .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));
    }

    // Test log content
    // The log should at least contains program version, engine name, address
    #[test]
    fn log_test() -> Result<()> {
        let temp_dir = tempdir()?;
        let stderr_path = temp_dir.path().join("stderr");
        let output = Command::cargo_bin("kvs-server")
            .unwrap()
            .args(&["--engine", "kvs", "--port", "4001"])
            .env("RUST_LOG", "debug")
            .current_dir(&temp_dir)
            .timeout(time::Duration::from_secs(1))
            .output()?;
        let mut err_file = File::create(&stderr_path)?;
        err_file.write_all(&output.stderr)?;

        String::from_utf8(output.stdout).expect("TODO: panic message");

        let content = fs::read_to_string(&stderr_path)?;
        assert!(content.contains(env!("CARGO_PKG_VERSION")));
        assert!(content.contains("kvs"));
        assert!(content.contains("127.0.0.1:4001"));
        Ok(())
    }

    fn cli_access_server(engine: &str, port: &str) -> Result<()> {
        let (tx, rx) = mpsc::sync_channel(0);
        let temp_dir = tempdir()?;
        let mut server = Command::cargo_bin("kvs-server").unwrap();
        server
            .args(&["--engine", engine, "--port", port])
            .current_dir(&temp_dir)
            .timeout(Duration::from_secs(2));
        let server_handle = thread::spawn(move || {
            let assert = server.assert();
            tx.send(assert).unwrap();
        });
        thread::sleep(Duration::from_secs(1));

        // Test set command, should success and output nothing
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "set", "key1", "value1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(str::is_empty());

        // Test get command, should get previously set str
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "get", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout("value1\n");

        // Test overwrite set
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "set", "key1", "value2"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(str::is_empty());

        // Test get after overwrite get
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "get", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout("value2\n");

        // Test get non-existent key
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "get", "key2"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout("\n");

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "rm", "key2"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(str::contains("Key not found"));

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "set", "key2", "value3"])
            .current_dir(&temp_dir)
            .assert()
            .success();

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "get", "key2"])
            .current_dir(&temp_dir)
            .assert()
            .stdout(str::contains("value3"));

        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "rm", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success();

        let assert = rx.recv().unwrap();

        // let (tx, rx) = mpsc::sync_channel(0);
        let mut server = Command::cargo_bin("kvs-server").unwrap();
        server
            .args(&["--engine", engine, "--port", port])
            .current_dir(&temp_dir)
            .timeout(Duration::from_secs(2));
        let server_handle = thread::spawn(move || {
            let assert = server.assert();
        });

        thread::sleep(Duration::from_secs(1));
        Command::cargo_bin("kvs-client")
            .unwrap()
            .args(&["--port", port, "get", "key2"])
            .current_dir(&temp_dir)
            .assert()
            .stdout(str::contains("value3"));
        server_handle.join().unwrap();

        Ok(())
    }

    #[test]
    fn cli_access_server_kvs_engine() -> Result<()>{
        cli_access_server("kvs", "6006")
    }
}