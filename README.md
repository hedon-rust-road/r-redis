# R-Redis 🧮

R-Redis is a simplified version of Redis implemented in Rust. It supports basic Redis commands such as `SET`, `GET`, `HSET`, `HGET`, `HMGET`, `HGETALL`, `ECHO`, `SADD`, and `SISMEMBER`. This project allows you to interact with it using the official `redis-cli`.

## Features ✨

- **SET**: Set the string value of a key.
- **GET**: Get the string value of a key.
- **HSET**: Set the string value of a field in a hash.
- **HGET**: Get the string value of a field in a hash.
- **HMGET**: Get the values of all the given fields in a hash.
- **HGETALL**: Get all the fields and values in a hash.
- **ECHO**: Echo the given string.
- **SADD**: Add one or more members to a set.
- **SISMEMBER**: Determine if a given value is a member of a set.

## Installation 🛠️

To get started with r-redis, you need to have Rust installed. You can install Rust using [rustup](https://rustup.rs/).

Clone the repository:

```bash
git clone https://github.com/hedon-rust-road/r-redis.git
cd r-redis
```

Build the project:

```bash
cargo build --release
```

Run the server:

```bash
./target/release/r-redis
```

## Usage 📚

Once the server is running, you can use the official `redis-cli` to interact with it:

```bash
redis-cli
```

You can now use the supported commands:

```bash
SET mykey "Hello, World!"
GET mykey
HSET myhash field1 "Hello"
HGET myhash field1
HMGET myhash field1 field2
HGETALL myhash
ECHO "Hello, World!"
SADD myset "Hello"
SISMEMBER myset "Hello"
```

## Core Crates 📦

- **bytes** (`1.6.0`): Utilities for working with bytes, used for efficient network communication.
- **dashmap** (`5.5.3`): A concurrent hashmap for efficient thread-safe access.
- **enum_dispatch** (`0.3.13`): Enables enum dispatch for dynamic command handling.
- **futures** (`0.3.30`): Asynchronous programming library.
- **lazy_static** (`1.4.0`): Allows for lazy initialization of static variables.
- **thiserror** (`1.0.61`): Simplifies error handling with custom error types.
- **tokio** (`1.37.0`): The asynchronous runtime for Rust, providing multi-threaded support and various utilities.
- **tokio-stream** (`0.1.15`): Stream utilities for Tokio.
- **tokio-util** (`0.7.11`): Utilities for working with Tokio, including codec support, to decode frames from tcp stream and encode frames to write to tcp stream.
- **tracing** (`0.1.40`): Instrumentation for application-level tracing.
- **tracing-subscriber** (`0.3.18`): Collects and records tracing data

## Acknowledgements 🙏

- [Rust Programming Language](https://www.rust-lang.org/)
- [Redis](https://redis.io/)

## License 📜

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
