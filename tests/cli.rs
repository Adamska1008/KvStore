mod cli_test {
    use assert_cmd::Command;
    use tempfile::TempDir;
    use kvs::Result;

    // Should exit with nonzero
    #[test]
    fn cli_no_args() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        Command::cargo_bin("kvs-client")
            .unwrap()
            .current_dir(&temp_dir)
            .assert()
            .failure();
        Ok(())
    }

    #[test]
    fn cli_invalid_get() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
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
}