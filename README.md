# OS-Interface

<p>
  <a href="LICENSE" target="_blank">
    <img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue.svg" />
  </a>
  <a href="LICENSE" target="_blank">
    <img alt="License: APACHE" src="https://img.shields.io/badge/License-APACHE-blue.svg" />
  </a>
  <a href="https://crates.io/crates/os-interface" target="_blank">
    <img src="https://img.shields.io/crates/v/os-interface.svg" />
  </a>
  <a href="https://crates.io/crates/os-interface" target="_blank">
    <img src="https://img.shields.io/crates/dr/os-interface" />
  </a>
  <a href="https://docs.rs/os-interface" target="_blank">
    <img src="https://docs.rs/os-interface/badge.svg" />
  </a>
</p>

<div align="center">

  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
  [![License](https://img.shields.io/badge/License-APACHE-blue.svg)](LICENSE-APACHE)
  [![Crates.io](https://img.shields.io/crates/v/os-interface.svg)](https://crates.io/crates/os-interface)
  [![Documentation](https://docs.rs/os-interface/badge.svg)](https://docs.rs/os-interface)
  ![Build](https://github.com/cmasus/os-interface/workflows/build/badge.svg)
  ![Clippy](https://github.com/cmasus/os-interface/workflows/clippy/badge.svg)
  ![Formatter](https://github.com/cmasus/os-interface/workflows/fmt/badge.svg)

</div>

<div align="center">

  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
  [![License: Apache 2.0](https://img.shields.io/badge/License-APACHE-blue.svg)](LICENSE-APACHE)
  [![Crates.io](https://img.shields.io/crates/v/os-interface.svg)](https://crates.io/crates/os-interface)
  [![Documentation](https://docs.rs/os-interface/badge.svg)](https://docs.rs/os-interface)
  [![Downloads](https://img.shields.io/crates/dr/os-interface)](https://crates.io/crates/os-interface)
  [![Rust Version](img.shields.io)](https://www.rust-lang.org)

</div>

Crate holding easy to use functions for retrieving information from the
operating system.

PR contributions are welcome. Contributions could be in the form of
improvements to existing functionality, adding new functionality or adding
support for other operating systems.

## Goals

* This crate aims to give an easy-to-use interface for retrieving data from the
OS, e.g. network information.
* Expand with more functionality as long as it adheres to above point.
* Have as few dependencies as possible. For example, in the Unix environments,
this crate only depends on libc which in turn does not depend on anything else.
* Encapsulate unsafe code.
* Return data in a Rust-like manner.

## Usage

Example usage

```rust
use os_interface::network::network_interfaces;

fn main() {
    let network_interfaces = network_interfaces().unwrap();

    println!("Network interfaces: {:#?}", network_interfaces);
}
```

gives an output similar to the following
```bash
Network interfaces: [
    NetworkInterface {
        index: 1,
        name: "lo",
        addr: [
            IPv4(
                IfAddrV4 {
                    ip: 127.0.0.1,
                    netmask: Some(
                        255.0.0.0,
                    ),
                    broadcast: None,
                },
            ),
            IPv6(
                IfAddrV6 {
                    ip: ::1,
                    netmask: Some(
                        ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff,
                    ),
                },
            ),
        ],
        mac_addr: Some(
            "00:00:00:00:00:00",
        ),
        flags: Flags {
            up: true,
            loopback: true,
            running: true,
            multicast: false,
            broadcast: false,
        },
    },
    ...
]
```

## Supported functions per operating system

function | linux | macos | windows | freebsd | android | ios
--- | --- | --- | --- | --- | --- | ---
network_interfaces() | yes | yes | no | yes | yes | yes
local_ipv4_addresses() | yes | yes | no | yes | yes | yes
local_ipv6_addresses() | yes | yes | no | yes | yes | yes
hostname() | yes | yes | no | yes | yes | yes
default_gateway() | yes | yes | no | yes | yes | yes

## API documentation
API documentation can be found [here](https://docs.rs/os-interface/)

## License

Distributed under the terms of both the MIT license and the Apache License
(Version 2.0)
