use crate::Error;
use libc::size_t;
use libc::{
    AF_INET, AF_INET6, IFF_BROADCAST, IFF_LOOPBACK, IFF_MULTICAST, IFF_RUNNING, IFF_UP,
    freeifaddrs, getifaddrs, if_nametoindex, ifaddrs, sockaddr_in, sockaddr_in6,
};
use std::collections::BTreeMap;
use std::ffi::{CStr, OsString};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::unix::ffi::OsStringExt;
use std::ptr;

/// System network interface
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NetworkInterface {
    /// Index
    pub index: u32,
    /// Name
    pub name: String, // e.g. eth0
    /// Address
    pub addr: Vec<Addr>,
    /// MAC address
    pub mac_addr: Option<String>,
    /// Interface flags
    pub flags: Flags,
}

/// Interface flags
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Flags {
    /// Interface is administratively up
    pub up: bool,
    /// Interface is a loopback device
    pub loopback: bool,
    /// Interface has resources allocated (operational)
    pub running: bool,
    /// Interface supports multicasting
    pub multicast: bool,
    /// Interface supports broadcast
    pub broadcast: bool,
}

/// Network interface address
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Addr {
    /// IPv4, AFINET address Family Internet Protocol version 4
    IPv4(IfAddrV4),
    /// IPv6, AFINET6 address Family Internet Protocol version 6
    IPv6(IfAddrV6),
}

/// IPv4 Interface from the AFINET network interface family
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IfAddrV4 {
    /// The IP address for this network interface
    pub ip: Ipv4Addr,
    /// The netmask for this interface
    pub netmask: Option<Ipv4Addr>,
    /// The broadcast address for this interface
    pub broadcast: Option<Ipv4Addr>,
}

/// IPv6 Interface from the AFINET6 network interface family
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IfAddrV6 {
    /// The IP address for this network interface
    pub ip: Ipv6Addr,
    /// The netmask for this interface
    pub netmask: Option<Ipv6Addr>,
}

fn if_addr_v4(ifa: &ifaddrs, flags: &Flags) -> IfAddrV4 {
    // Get Netmask
    let mut netmask: Option<Ipv4Addr> = None;
    if !ifa.ifa_netmask.is_null() {
        let mask = unsafe { *(ifa.ifa_netmask as *const sockaddr_in) };
        // Netmask in network byte order (Big Endian)
        netmask = Some(Ipv4Addr::from(mask.sin_addr.s_addr.to_be()));
    }

    // Access the broadcast address from the union field
    // Note: In some libc versions, this is called ifa_ifu; in others, ifa_dstaddr
    // Use cfg to handle the field name difference
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let broad_ptr = ifa.ifa_ifu;

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
    let broad_ptr = ifa.ifa_dstaddr;

    let mut broadcast: Option<Ipv4Addr> = None;
    // Check if the interface supports broadcasting
    if !broad_ptr.is_null() && flags.broadcast {
        let sa = unsafe { *(broad_ptr as *const sockaddr_in) };
        broadcast = Some(Ipv4Addr::from(u32::from_be(sa.sin_addr.s_addr)));
    }

    // Get address
    let socket_addr = unsafe { *(ifa.ifa_addr as *const sockaddr_in) };
    let ip = Ipv4Addr::from(socket_addr.sin_addr.s_addr.to_be());

    IfAddrV4 {
        ip,
        netmask,
        broadcast,
    }
}

fn if_addr_v6(ifa: &ifaddrs) -> IfAddrV6 {
    let mut netmask: Option<Ipv6Addr> = None;
    if !ifa.ifa_netmask.is_null() {
        let mask = unsafe { *(ifa.ifa_netmask as *const sockaddr_in6) };
        // Access 16-byte array for IPv6 netmask
        netmask = Some(Ipv6Addr::from(mask.sin6_addr.s6_addr));
    }

    let socket_addr = unsafe { *(ifa.ifa_addr as *const sockaddr_in6) };
    let ip = Ipv6Addr::from(socket_addr.sin6_addr.s6_addr);

    IfAddrV6 { ip, netmask }
}

fn mac_to_string(mac: &[u8]) -> String {
    let mac_addr = format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    mac_addr
}

fn mac_addr(ifa: &ifaddrs, family: i32) -> Option<String> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    if family == libc::AF_PACKET {
        let sll = unsafe { *(ifa.ifa_addr as *const libc::sockaddr_ll) };
        let mac = sll.sll_addr;
        let len = sll.sll_halen as usize;

        // MAC addresses are usually 6 bytes (Ethernet)
        if len == 6 {
            return Some(mac_to_string(&mac));
        }
    }

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
    if family == libc::AF_LINK {
        let sdl = unsafe { *(ifa.ifa_addr as *const libc::sockaddr_dl) };

        // Use LLADDR macro logic: it skips the name (sdl_nlen)
        // to find the start of the hardware address
        let mac_ptr = unsafe { (&sdl as *const _ as *const u8).add(8 + sdl.sdl_nlen as usize) };

        if sdl.sdl_alen == 6 {
            let mac = unsafe { std::slice::from_raw_parts(mac_ptr, 6) };
            return Some(mac_to_string(mac));
        }
    }

    None
}

/// Inserts a new NetworkInterface into the BTreeMap
/// or updates with another address in the addr list.
fn update_interfaces(
    index: u32,
    name: String,
    addr: Addr,
    flags: Flags,
    interfaces: &mut BTreeMap<u32, NetworkInterface>,
) {
    interfaces
        .entry(index)
        .and_modify(|i| {
            i.addr.push(addr);
        })
        .or_insert(NetworkInterface {
            index,
            name,
            addr: vec![addr],
            mac_addr: None,
            flags,
        });
}

/// Inserts a new NetworkInterface into the BTreeMap
/// or updates the mac address for the given NetworkInterface.
fn update_interfaces_with_mac(
    index: u32,
    name: String,
    mac_addr: Option<String>,
    flags: Flags,
    interfaces: &mut BTreeMap<u32, NetworkInterface>,
) {
    interfaces
        .entry(index)
        .and_modify(|i| {
            i.mac_addr = mac_addr.clone();
        })
        .or_insert(NetworkInterface {
            index,
            name,
            addr: Vec::new(),
            mac_addr,
            flags,
        });
}

/// This function exist for backward compatibility.
/// Use network_interfaces() instead.
pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>, Error> {
    network_interfaces()
}

/// Get all the network interfaces.
pub fn network_interfaces() -> Result<Vec<NetworkInterface>, Error> {
    let mut ifaddr_ptr: *mut ifaddrs = ptr::null_mut();

    unsafe {
        // Retrieve the linked list of interfaces
        let res = getifaddrs(&mut ifaddr_ptr);
        if res != 0 {
            return Err(Error::FailedToGetResource(format!(
                "getifaddrs returned {res}"
            )));
        }
    }

    let mut interfaces: BTreeMap<u32, NetworkInterface> = BTreeMap::new();

    let mut current_ptr = ifaddr_ptr;
    while let Some(ifa) = unsafe { current_ptr.as_ref() } {
        // Extract the interface name
        let name = unsafe { CStr::from_ptr(ifa.ifa_name).to_string_lossy() };

        // Extract interface index
        let index = unsafe { if_nametoindex(ifa.ifa_name) };
        if index == 0 {
            // Returns 0 on failure (e.g., interface no longer exists)
            eprint!("Interface no longer exists: {name}");
        }

        // Extract interface flags
        let flags = ifa.ifa_flags;
        let flags = Flags {
            up: (flags as i32 & IFF_UP) != 0,
            loopback: (flags as i32 & IFF_LOOPBACK) != 0,
            running: (flags as i32 & IFF_RUNNING) != 0,
            multicast: (flags as i32 & IFF_MULTICAST) != 0,
            broadcast: (flags as i32 & IFF_BROADCAST) != 0,
        };

        // Process the address if it exists
        let Some(ifa_addr) = (unsafe { ifa.ifa_addr.as_ref() }) else {
            current_ptr = ifa.ifa_next;
            continue;
        };
        let family = ifa_addr.sa_family as i32;

        match family {
            AF_INET => {
                let if_addr_v4 = if_addr_v4(ifa, &flags);
                let addr = Addr::IPv4(if_addr_v4);
                update_interfaces(index, name.into_owned(), addr, flags, &mut interfaces);
            }
            AF_INET6 => {
                let if_addr_v6 = if_addr_v6(ifa);
                let addr = Addr::IPv6(if_addr_v6);
                update_interfaces(index, name.into_owned(), addr, flags, &mut interfaces);
            }
            family => {
                let mac_addr = mac_addr(ifa, family);
                update_interfaces_with_mac(
                    index,
                    name.into_owned(),
                    mac_addr,
                    flags,
                    &mut interfaces,
                );
            }
        }

        current_ptr = ifa.ifa_next;
    }

    unsafe {
        freeifaddrs(ifaddr_ptr);
    }

    Ok(interfaces.into_values().collect())
}

/// Gets all local IPv4 addresses that are not loopback.
pub fn local_ipv4_addresses() -> Result<Vec<Ipv4Addr>, Error> {
    Ok(network_interfaces()?
        .into_iter()
        .filter_map(|ni| {
            if !ni.flags.loopback {
                ni.addr.into_iter().find_map(|addr| match addr {
                    Addr::IPv4(addr) => Some(addr.ip),
                    _ => None,
                })
            } else {
                None
            }
        })
        .collect())
}

/// Gets all local IPv6 addresses that are not loopback or unicast link local.
pub fn local_ipv6_addresses() -> Result<Vec<Ipv6Addr>, Error> {
    Ok(network_interfaces()?
        .into_iter()
        .filter_map(|ni| {
            if !ni.flags.loopback {
                ni.addr.into_iter().find_map(|addr| match addr {
                    Addr::IPv6(addr) if !addr.ip.is_unicast_link_local() => Some(addr.ip),
                    _ => None,
                })
            } else {
                None
            }
        })
        .collect())
}

/// This function exist for backward compatibility.
/// Use hostname() instead.
pub fn get_hostname() -> Result<OsString, Error> {
    hostname()
}

/// Get the hostname.
pub fn hostname() -> Result<OsString, Error> {
    let mut buf: Vec<u8> = Vec::with_capacity(256);
    let ptr = buf.as_mut_ptr().cast();
    let len = buf.capacity() as size_t;

    let res = unsafe { libc::gethostname(ptr, len) };
    if res != 0 {
        return Err(Error::FailedToGetResource(format!(
            "gethostname returned {res}"
        )));
    }
    unsafe {
        buf.as_mut_ptr().wrapping_add(len - 1).write(0);
        let len = CStr::from_ptr(buf.as_ptr().cast()).count_bytes();
        buf.set_len(len);
    }
    Ok(OsString::from_vec(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_interfaces() {
        let interfaces = network_interfaces().expect("Failed to get network interfaces");
        println!("Interfaces: {interfaces:#?}");
        assert!(interfaces.len() > 0);
        assert!(interfaces[0].name.starts_with("lo"));
        assert!(interfaces[0].addr.len() > 0);
    }

    #[test]
    fn test_local_ipv4_addresses() {
        let addresses = local_ipv4_addresses().expect("Failed to get IPv4 addresses");
        assert!(addresses.len() >= 1);
    }

    #[test]
    fn test_local_ipv6_addresses() {
        let addresses = local_ipv6_addresses();
        assert!(addresses.is_ok());
    }

    #[test]
    fn test_hostname() {
        let hostname = hostname().expect("Failed to get hostname");
        println!("hostname: {hostname:#?}");
        assert!(hostname.len() > 0);
    }
}
