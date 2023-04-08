# KvStore
A simple key-value database built under the [guidance](https://github.com/pingcap/talent-plan/tree/master/courses/rust/projects/project-2) of [talent-plan](https://github.com/pingcap/talent-plan).

## Installation

### Prerequisite
This binary use several unstable features of rust. To build it, using nightly version of rustc.

### cargo build
In the root directory, run `cargo build` to build the program.

### Serde-Resp
The project depends [`Serde-Resp`](https://github.com/Adamska1008/Serde-Resp)(another project of mine).

## Usage
There are two binaries:
* `kvs-client`: the client of database
* `kvs-server`: the server of database

### kvs-server
This program run a server, listen to a binding port and ready for connections. The messages delivered are in RESP format and use this repository([link](https://github.com/Adamska1008/Serde-Resp)).

`./kvs-server` will run the binary in default set. There are two options available:
* `-p <PORT>` or `--port <PORT>`, set the listened port. Default `4000`.
* `-e <ENGINE>` or `--engine <ENGINE>`, set the engine of database. The database supports two engines:
  * `kvs`, self written engine, default.
  * `sled`, use the api of sled([link](https://github.com/spacejam/sled)).

use `--help` to see the detail.
```bash
Demo program that demonstrates the usage of "KvStore" core

Usage: kvs-server.exe [OPTIONS]

Options:
  -p, --port <PORT>
  -e, --engine <ENGINE>  [possible values: kvs, sled]
  -h, --help             Print help
  -V, --version          Print version
```


### kvs-client
Currently, it supports only few commands:
* `set <KEY> <VALUE>`: Store a key-value pair to database.
* `get <KEY>`: Get value of key from database. Exit with non-zero if `<KEY>` is not in database.
* `rm <KEY>`: Remove a key-value pair with given key. Exit with non-zero if `<KEY>` is not in database.
* `-p --port <PORT>`: The connecting port, default `4000`.

Use `--help` to see the detail.
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

### Use as package
To use `KvStore` in your code(not advised, this is simply a student coding practise), `import kvs::engine::KvStore`.

For more information, run `cargo doc --open` to see the document of `KvStore`.

## Comparison to talent-plan standard code
* The store methods accept `&str` instead of `String` as args.
* The `rm` method will return `Some(())` when found key, and `None` when key is not found.
* Client will produce empty output when get a non-existent key, instead of "Key not found" which may be confused when the value is "Key not found".
* Client will not exit with error when remove a non-existent key.
