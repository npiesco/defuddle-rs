*[TokioConf 2026 program and tickets are now available!](https://tokioconf.com/)*

---

## Tokio

A runtime for writing reliable, asynchronous, and slim applications with the Rust programming language. It is:

- **Fast**: Tokio's zero-cost abstractions give you bare-metal performance.
- **Reliable**: Tokio leverages Rust's ownership, type system, and concurrency model to reduce bugs and ensure thread safety.
- **Scalable**: Tokio has a minimal footprint, and handles backpressure and cancellation naturally.

[![Crates.io](https://camo.githubusercontent.com/7a0af26d342f28b312ac93e3e24fee7cbd1c0c1b4a609dc6cacc4f5197f67b76/68747470733a2f2f696d672e736869656c64732e696f2f6372617465732f762f746f6b696f2e737667)](https://crates.io/crates/tokio) [![MIT licensed](https://camo.githubusercontent.com/7013272bd27ece47364536a221edb554cd69683b68a46fc0ee96881174c4214c/68747470733a2f2f696d672e736869656c64732e696f2f62616467652f6c6963656e73652d4d49542d626c75652e737667)](https://github.com/tokio-rs/tokio/blob/master/LICENSE) [![Build Status](https://github.com/tokio-rs/tokio/workflows/CI/badge.svg)](https://github.com/tokio-rs/tokio/actions?query=workflow%3ACI+branch%3Amaster) [![Discord chat](https://camo.githubusercontent.com/2596d83d04909cf4c0846071c0f92365991a5f9ccf9cd36c9c17d574f0142b55/68747470733a2f2f696d672e736869656c64732e696f2f646973636f72642f3530303032383838363032353839353933362e7376673f6c6f676f3d646973636f7264267374796c653d666c61742d737175617265)](https://discord.gg/tokio)

[Website](https://tokio.rs/) | [Guides](https://tokio.rs/tokio/tutorial) | [API Docs](https://docs.rs/tokio/latest/tokio) | [Chat](https://discord.gg/tokio)

## Overview

Tokio is an event-driven, non-blocking I/O platform for writing asynchronous applications with the Rust programming language. At a high level, it provides a few major components:

- A multithreaded, work-stealing based task [scheduler](https://docs.rs/tokio/latest/tokio/runtime/index.html).
- A reactor backed by the operating system's event queue (epoll, kqueue, IOCP, etc.).
- Asynchronous [TCP and UDP](https://docs.rs/tokio/latest/tokio/net/index.html) sockets.

These components provide the runtime components necessary for building an asynchronous application.

## Example

A basic TCP echo server with Tokio.

Make sure you enable the full features of the tokio crate on Cargo.toml:

```
[dependencies]
tokio = { version = "1.51.0", features = ["full"] }
```

Then, on your main.rs:

```
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
```

More examples can be found [here](https://github.com/tokio-rs/tokio/tree/master/examples). For a larger "real world" example, see the [mini-redis](https://github.com/tokio-rs/mini-redis/) repository.

To see a list of the available feature flags that can be enabled, check our [docs](https://docs.rs/tokio/#feature-flags).

## Getting Help

First, see if the answer to your question can be found in the [Guides](https://tokio.rs/tokio/tutorial) or the [API documentation](https://docs.rs/tokio/latest/tokio). If the answer is not there, there is an active community in the [Tokio Discord server](https://discord.gg/tokio). We would be happy to try to answer your question. You can also ask your question on [the discussions page](https://github.com/tokio-rs/tokio/discussions).

## Contributing

🎈 Thanks for your help improving the project! We are so happy to have you! We have a [contributing guide](https://github.com/tokio-rs/tokio/blob/master/docs/contributing/README.md) to help you get involved in the Tokio project.

## Related Projects

In addition to the crates in this repository, the Tokio project also maintains several other libraries, including:

- [`axum`](https://github.com/tokio-rs/axum): A web application framework that focuses on ergonomics and modularity.
- [`hyper`](https://github.com/hyperium/hyper): A fast and correct HTTP/1.1 and HTTP/2 implementation for Rust.
- [`tonic`](https://github.com/hyperium/tonic): A gRPC over HTTP/2 implementation focused on high performance, interoperability, and flexibility.
- [`warp`](https://github.com/seanmonstar/warp): A super-easy, composable, web server framework for warp speeds.
- [`tower`](https://github.com/tower-rs/tower): A library of modular and reusable components for building robust networking clients and servers.
- [`tracing`](https://github.com/tokio-rs/tracing) (formerly `tokio-trace`): A framework for application-level tracing and async-aware diagnostics.
- [`mio`](https://github.com/tokio-rs/mio): A low-level, cross-platform abstraction over OS I/O APIs that powers `tokio`.
- [`bytes`](https://github.com/tokio-rs/bytes): Utilities for working with bytes, including efficient byte buffers.
- [`loom`](https://github.com/tokio-rs/loom): A testing tool for concurrent Rust code.

## Changelog

The Tokio repository contains multiple crates. Each crate has its own changelog.

- `tokio` - [view changelog](https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md)
- `tokio-util` - [view changelog](https://github.com/tokio-rs/tokio/blob/master/tokio-util/CHANGELOG.md)
- `tokio-stream` - [view changelog](https://github.com/tokio-rs/tokio/blob/master/tokio-stream/CHANGELOG.md)
- `tokio-macros` - [view changelog](https://github.com/tokio-rs/tokio/blob/master/tokio-macros/CHANGELOG.md)
- `tokio-test` - [view changelog](https://github.com/tokio-rs/tokio/blob/master/tokio-test/CHANGELOG.md)

## Supported Rust Versions

Tokio will keep a rolling MSRV (minimum supported rust version) policy of **at least** 6 months. When increasing the MSRV, the new Rust version must have been released at least six months ago. The current MSRV is 1.71.

Note that the MSRV is not increased automatically, and only as part of a minor release. The MSRV history for past minor releases can be found below:

- 1.48 to now - Rust 1.71
- 1.39 to 1.47 - Rust 1.70
- 1.30 to 1.38 - Rust 1.63
- 1.27 to 1.29 - Rust 1.56
- 1.17 to 1.26 - Rust 1.49
- 1.15 to 1.16 - Rust 1.46
- 1.0 to 1.14 - Rust 1.45

Note that although we try to avoid the situation where a dependency transitively increases the MSRV of Tokio, we do not guarantee that this does not happen. However, every minor release will have some set of versions of dependencies that works with the MSRV of that minor release.

## Release schedule

Tokio doesn't follow a fixed release schedule, but we typically make one minor release each month. We make patch releases for bugfixes as necessary.

## Bug patching policy

For the purposes of making patch releases with bugfixes, we have designated certain minor releases as LTS (long term support) releases. Whenever a bug warrants a patch release with a fix for the bug, it will be backported and released as a new patch release for each LTS minor version. Our current LTS releases are:

- `1.47.x` - LTS release until September 2026. (MSRV 1.70)
- `1.51.x` - LTS release until March 2027. (MSRV 1.71)

Each LTS release will continue to receive backported fixes for at least a year. If you wish to use a fixed minor release in your project, we recommend that you use an LTS release.

To use a fixed minor version, you can specify the version with a tilde. For example, to specify that you wish to use the newest `1.47.x` patch release, you can use the following dependency specification:

```
tokio = { version = "~1.47", features = [...] }
```

### Previous LTS releases

- `1.8.x` - LTS release until February 2022.
- `1.14.x` - LTS release until June 2022.
- `1.18.x` - LTS release until June 2023.
- `1.20.x` - LTS release until September 2023.
- `1.25.x` - LTS release until March 2024.
- `1.32.x` - LTS release until September 2024.
- `1.36.x` - LTS release until March 2025.
- `1.38.x` - LTS release until July 2025.
- `1.43.x` - LTS release until March 2026.

## License

This project is licensed under the [MIT license](https://github.com/tokio-rs/tokio/blob/master/LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Tokio by you shall be licensed as MIT, without any additional terms or conditions.
