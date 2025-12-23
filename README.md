# OS-Interface

Crate holding easy to use functions for retrieving information from the
operating system.

PR contributions are welcome. Contributions could be in the form of
improvements to existing functionality, adding support for other operating
systems or adding new functionality.

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
use os_interface::network::get_network_interfaces;

fn main() {
    let network_interfaces = get_network_interfaces().unwrap();

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
get_network_interfaces() | yes | yes | no | yes | yes | yes
get_hostname() | yes | yes | no | yes | yes | yes

<!---
## API documentation
API documentation can be found [here](https://docs.rs/os-interface/)
--->

## License

Distributed under the terms of both the MIT license and the Apache License
(Version 2.0)
