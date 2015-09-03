use libc::{c_int, pid_t, uid_t};
use {Error, Result};
use unistd;
use errno::Errno;
use sys::signal::signal::siginfo as signal_siginfo;
pub use sys::signal::{self, SigSet};

use std::os::unix::io::RawFd;
use std::mem;

mod ffi {
    use libc::c_int;
    use sys::signal::sigset_t;

    extern {
        pub fn signalfd(fd: c_int, mask: *const sigset_t, flags: c_int) -> c_int;
    }
}

bitflags!{
    flags SfdFlags: c_int {
        const SFD_NONBLOCK  = 0o00004000, // O_NONBLOCK
        const SFD_CLOEXEC   = 0o02000000, // O_CLOEXEC
    }
}

pub const CREATE_NEW_FD: RawFd = -1;

/// Creates a new file descriptor for reading signals.
///
/// The `mask` parameter specifies the set of signals that can be accepted via this file descriptor.
///
/// See [the signalfd man page for more information](http://man7.org/linux/man-pages/man2/signalfd.2.html)
pub fn signalfd(fd: RawFd, mask: &SigSet, flags: SfdFlags) -> Result<RawFd> {
    unsafe {
        match ffi::signalfd(fd as c_int, mask.as_ref(), flags.bits()) {
            -1 => Err(Error::Sys(Errno::last())),
            res => Ok(res as RawFd),
        }
    }
}

#[derive(Debug)]
pub struct SignalFd(RawFd);

impl SignalFd {
    pub fn new(mask: &SigSet) -> Result<SignalFd> {
        Self::with_flags(mask, SfdFlags::empty())
    }

    pub fn with_flags(mask: &SigSet, flags: SfdFlags) -> Result<SignalFd> {
        let fd = try!(signalfd(CREATE_NEW_FD, mask, flags));

        Ok(SignalFd(fd))
    }

    pub fn set_mask(&mut self, mask: &SigSet) -> Result<()> {
        signalfd(self.0, mask, SfdFlags::empty()).map(|_| ())
    }

    pub fn read_signal(&mut self) -> Result<Option<siginfo>> {
        let mut buffer: [u8; SIGINFO_SIZE] = unsafe { mem::uninitialized() };

        match unistd::read(self.0, &mut buffer) {
            Ok(SIGINFO_SIZE) => Ok(Some(unsafe { mem::transmute_copy(&buffer) })),
            Ok(_) => unreachable!("partial read on signalfd"),
            Err(Error::Sys(Errno::EAGAIN)) => Ok(None),
            Err(error) => Err(error)
        }
    }
}

impl Drop for SignalFd {
    fn drop(&mut self) {
        let _ = unistd::close(self.0);
    }
}

impl Iterator for SignalFd {
    type Item = siginfo;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_signal() {
            Ok(Some(sig)) => Some(sig),
            Ok(None) => None,
            Err(..) => None,
        }
    }
}

pub const SIGINFO_SIZE: usize = 128;
pub const SIGINFO_PADDING: usize = 48;

#[derive(Debug, Clone, PartialEq)]
#[repr(C, packed)]
pub struct siginfo {
    pub ssi_signo: u32,
    pub ssi_errno: i32,
    pub ssi_code: i32,
    pub ssi_pid: u32,
    pub ssi_uid: u32,
    pub ssi_fd: i32,
    pub ssi_tid: u32,
    pub ssi_band: u32,
    pub ssi_overrun: u32,
    pub ssi_trapno: u32,
    pub ssi_status: i32,
    pub ssi_int: i32,
    pub ssi_ptr: u64,
    pub ssi_utime: u64,
    pub ssi_stime: u64,
    pub ssi_addr: u64,
}

impl Into<signal_siginfo> for siginfo {
    fn into(self) -> signal_siginfo {
        signal_siginfo {
            si_signo: self.ssi_signo as c_int,
            si_errno: self.ssi_errno as c_int,
            si_code: self.ssi_code as c_int,
            pid: self.ssi_pid as pid_t,
            uid: self.ssi_uid as uid_t,
            status: self.ssi_status as c_int,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn check_siginfo_size() {
        assert_eq!(mem::size_of::<siginfo>() + SIGINFO_PADDING, SIGINFO_SIZE);
    }

    #[test]
    fn create_signalfd() {
        let mask = SigSet::empty();
        let fd = SignalFd::new(&mask);
        assert!(fd.is_ok());
    }

    #[test]
    fn create_signalfd_with_opts() {
        let mask = SigSet::empty();
        let fd = SignalFd::with_flags(&mask, SFD_CLOEXEC | SFD_NONBLOCK);
        assert!(fd.is_ok());
    }

    #[test]
    fn read_empty_signalfd() {
        let mask = SigSet::empty();
        let mut fd = SignalFd::with_flags(&mask, SFD_NONBLOCK).unwrap();

        let res = fd.read_signal();
        assert_eq!(res, Ok(None));
    }
}
