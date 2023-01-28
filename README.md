# Warning: Switch to `core` branch
The README.md file is applied to `core` branch. In current branch I'm try to build a c/s program based on the core and the README here is not applicable.

# KvStore
A simple key-value database built under the [guidance](https://github.com/pingcap/talent-plan/tree/master/courses/rust/projects/project-2) of [talent-plan](https://github.com/pingcap/talent-plan).

## Installation

### Prerequisite
This binary use several unstable features of rust. To build it, using nightly version of rustc.

### cargo build
In the root directory, run `cargo build` to build the program.

## Usage
Currently, it supports only few commands:
* `Kvs set <KEY> <VALUE>`: Store a key-value pair to database.
* `Kvs get <KEY>`: Get value of key from database. Exit with non-zero if `<KEY>` is not in database.
* `Kvs rm <KEY>`: Remove a key-value pair with given key. Exit with non-zero if `<KEY>` is not in database.

### Options
* `-h --help` display help information like:
```bash
Demo program that demonstrates the usage of "KvStore" core.

Usage: Kvs.exe [COMMAND]

Commands:
  get   get string value from kv store with given key
  set   set key-value string pair into kv store
  rm    remove key-value string from kv store with given key
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```
* `-V --version` Print version information

### Use as package
To use `KvStore` in your code(not advised, this is simply a personal coding practise), `import Kvs::KvStore`.

For more information, run `cargo doc --open` to see the document of `KvStore`.

## Comparison to talent-plan standard code
* The store methods accept `&str` instead of `String` as args.
* The `rm` method will return `Some(())` when found key, and `None` when key is not found.
* Client will produce empty output when get a non-existent key, instead of "Key not found" which may be confused when the value is "Key not found".
* Client will not exit with error when remove a non-existent key.