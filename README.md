# KvStore
A simple key-value database built under the [guidance](https://github.com/pingcap/talent-plan/tree/master/courses/rust/projects/project-2) of [talent-plan](https://github.com/pingcap/talent-plan).

## Installation

### Prerequisite
This binary use several unstable features of rust. To build it, using nightly version of rustc.

### cargo build
In the root directory, run `cargo build` to build the program.

## Usage
Currently, it supports only few commands:
* `kvs set <KEY> <VALUE>`: Store a key-value pair to database.
* `kvs get <KEY>`: Get value of key from database. Exit with non-zero if `<KEY>` is not in database.
* `kvs rm <KEY>`: Remove a key-value pair with given key. Exit with non-zero if `<KEY>` is not in database.

### Options
* `-h --help` display help information like:
```bash
Demo program that demonstrates the usage of "KvStore" core.

Usage: kvs.exe [COMMAND]

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

For more information, run `cargo doc --open` to see the document of `KvStore`.