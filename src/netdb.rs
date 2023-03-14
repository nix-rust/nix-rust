//! Safe wrappers around functions found in POSIX <netdb.h> header
//! 
//! https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netdb.h.html
use std::fmt::Debug;

use crate::errno::Errno;
use crate::sys::socket::AddressFamily;

// The <netdb.h> header may define the in_port_t type and the in_addr_t type as described in <netinet/in.h>.
// Simple integer type aliases, so we rexport
pub use libc::{in_port_t, in_addr_t};

/// Corresponds to `addrinfo`.
/// Deliberately is not Clone because we want to own indirect data.
#[repr(transparent)]
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct AddrInfo(libc::addrinfo);
impl AddrInfo {
    /// `ai_flags` Input flags. 
    pub fn flags(&self) -> Option<AiFlags> {
        AiFlags::from_bits(self.0.ai_flags)
    }
    /// `ai_flags`: set input flags. 
    pub fn set_flags(&mut self, flags: AiFlags) {
        self.0.ai_flags = flags.bits();
    }
    /// `ai_family`: Address family of socket.
    pub fn family(&self) -> Option<AddressFamily> {
        AddressFamily::from_i32(self.0.ai_family)
    }
    /// `ai_family`: set address family of socket.
    pub fn set_family(&mut self, family: AddressFamily) {
        self.0.ai_family = family as _;
    }
    // int               ai_socktype   Socket type. 
    // int               ai_protocol   Protocol of socket. 
    // socklen_t         ai_addrlen    Length of socket address. 
    // struct sockaddr  *ai_addr       Socket address of socket. 
    // char             *ai_canonname  Canonical name of service location. 
    /// Pointer to next in list. 
    pub fn next(&self) -> Option<&Self> {
        // SAFETY: we are properly initialized and are propagating our lifetime
        unsafe { self.0.ai_next.cast::<Self>().as_ref() }
    }
    /// Mutable pointer to next in list. 
    pub fn next_mut(&mut self) -> Option<&mut Self> {
        // SAFETY: we are properly initialized and are propagating our lifetime
        unsafe { self.0.ai_next.cast::<Self>().as_mut() }
    } 
}

libc_bitflags!{
    ///  the flags field of the addrinfo structure
    pub struct AiFlags: libc::c_int {
        /// Socket address is intended for bind().
        AI_PASSIVE;
        /// Request for canonical name.
        AI_CANONNAME;
        /// Return numeric host address as name.
        AI_NUMERICHOST;
        /// Inhibit service name resolution.
        AI_NUMERICSERV;
        /// If no IPv6 addresses are found,
        /// query for IPv4 addresses and return them to the caller as IPv4-mapped IPv6 addresses.
        AI_V4MAPPED;
        /// Query for both IPv4 and IPv6 addresses.
        AI_ALL;
        /// Query for IPv4 addresses only when an IPv4 address is configured;
        /// Query for IPv6 addresses only when an IPv6 address is configured.
        AI_ADDRCONFIG;
    }
}

libc_bitflags!{
    ///  the flags argument to getnameinfo():
    pub struct NiFlags: libc::c_int {
        /// Only the nodename portion of the FQDN is returned for local hosts.
        NI_NOFQDN;
        /// The numeric form of the node's address is returned instead of its name.
        NI_NUMERICHOST;
        /// Return an error if the node's name cannot be located in the database.
        NI_NAMEREQD;
        /// The numeric form of the service address is returned instead of its name.
        NI_NUMERICSERV;
        /// For IPv6 addresses, the numeric form of the scope identifier is returned instead of its name.
        NI_NUMERICSCOPE;
        /// Indicates that the service is a datagram service (SOCK_DGRAM).
        NI_DGRAM;
    }
}

/// error values for getaddrinfo() and getnameinfo().
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
#[non_exhaustive]
pub enum AddressInfoError {
    // UnknownErrno = 0,
    /// The name could not be resolved at this time. Future attempts may succeed.
    EAI_AGAIN = libc::EAI_AGAIN,
    /// The flags had an invalid value.
    EAI_BADFLAGS = libc::EAI_BADFLAGS,
    /// A non-recoverable error occurred.
    EAI_FAIL = libc::EAI_FAIL,
    /// The address family was not recognized or the address length was invalid for the specified family.
    EAI_FAMILY = libc::EAI_FAMILY,
    /// There was a memory allocation failure.
    EAI_MEMORY = libc::EAI_MEMORY,
    /// The name does not resolve for the supplied parameters.
    /// NI_NAMEREQD is set and the host's name cannot be located, or both nodename and servname were null.
    EAI_NONAME = libc::EAI_NONAME,
    /// The service passed was not recognized for the specified socket type.
    EAI_SERVICE = libc::EAI_SERVICE,
    /// The intended socket type was not recognized.
    EAI_SOCKTYPE = libc::EAI_SOCKTYPE,
    /// A system error occurred. The error code can be found in errno.
    EAI_SYSTEM(Errno) = libc::EAI_SYSTEM,
    /// An argument buffer overflowed.
    EAI_OVERFLOW = libc::EAI_OVERFLOW,
}
pub use libc::socklen_t;