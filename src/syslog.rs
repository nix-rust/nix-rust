//! Interfaces for controlling system log.

use crate::NixPath;
use crate::Result;
use std::ffi::OsStr;

/// Logging options of subsequent [`syslog`] calls can be set by calling [`openlog`].
///
/// The parameter `ident` is a string that will be prepended to every message. The `logopt`
/// argument specifies logging options. The `facility` parameter encodes a default facility to be
/// assigned to all messages that do not have an explicit facility encoded.
pub fn openlog<P: NixPath + ?Sized>(
    ident: &P,
    logopt: LogFlags,
    facility: Facility,
) -> Result<()> {
    ident.with_nix_path(|ident| unsafe {
        libc::openlog(ident.as_ptr(), logopt.bits(), facility as libc::c_int);
    })
}

/// Writes message to the system message logger.
///
/// The message is then written to the system console, log files, logged-in users, or forwarded
/// to other machines as appropriate.
pub fn syslog<S: AsRef<OsStr> + ?Sized>(priority: Priority, message: &S) {
    let formatter = OsStr::new("%s");
    let message = OsStr::new(message);
    unsafe { libc::syslog(priority.0, formatter.as_ptr(), message.as_ptr()) }
}

/// The priority for a log message.
#[derive(Debug, Clone, Copy)]
pub struct Priority(libc::c_int);

impl Priority {
    /// Create a new priority from a severity level.
    pub fn from_severity(severity: Severity) -> Self {
        let priority = severity as libc::c_int;
        Priority(priority)
    }

    /// Create a new priority from a facility and severity level.
    pub fn from(severity: Severity, facility: Facility) -> Self {
        let priority = (facility as libc::c_int) | (severity as libc::c_int);
        Priority(priority)
    }
}

libc_bitflags! {
    pub struct LogFlags: libc::c_int {
        /// Log the process id with each message: useful for identifying instantiations of
        /// daemons.
        LOG_PID;
        /// If syslog() cannot pass the message to syslogd(8) it will attempt to write the
        /// message to the console ("/dev/console").
        LOG_CONS;
        /// Open the connection to syslogd(8) immediately. Normally the open is delayed until
        /// the first message is logged. Useful for programs that need to manage the order in
        /// which file descriptors are allocated.
        LOG_NDELAY;
        /// Write the message to standard error output as well to the system log.
        LOG_PERROR;
    }
}

libc_enum! {
    /// Severity levels for log messages.
    #[repr(i32)]
    #[non_exhaustive]
    pub enum Severity {
        /// A panic condition.
        ///
        /// This is normally broadcast to all users.
        LOG_EMERG,
        /// A condition that should be corrected immediately, such as a corrupted system database.
        LOG_ALERT,
        /// Critical conditions, e.g., hard device errors.
        LOG_CRIT,
        /// Errors.
        LOG_ERR,
        /// Warning messages.
        LOG_WARNING,
        /// Conditions that are not error conditions, but should possibly be handled specially.
        LOG_NOTICE,
        /// Informational messages.
        LOG_INFO,
        /// Messages that contain information normally of use only when debugging a program.
        LOG_DEBUG,
    }
}

libc_enum! {
    /// Facilities for log messages.
    #[repr(i32)]
    #[non_exhaustive]
    pub enum Facility {
        /// Messages generated by the kernel.
        ///
        /// These cannot be generated by any user processes.
        LOG_KERN,
        /// Messages generated by random user processes.
        ///
        /// This is the default facility identifier if none is specified.
        LOG_USER,
        /// The mail system.
        LOG_MAIL,
        /// System daemons, such as routed(8), that are not provided for explicitly by other facilities.
        LOG_DAEMON,
        /// The authorization system: login(1), su(1), getty(8), etc.
        LOG_AUTH,
        /// Messages generated internally by syslogd(8).
        LOG_SYSLOG,
        /// The line printer spooling system: cups-lpd(8), cupsd(8), etc.
        LOG_LPR,
        /// The network news system.
        LOG_NEWS,
        /// The uucp system.
        LOG_UUCP,
        /// Reserved for local use.
        LOG_LOCAL0,
        /// Reserved for local use.
        LOG_LOCAL1,
        /// Reserved for local use.
        LOG_LOCAL2,
        /// Reserved for local use.
        LOG_LOCAL3,
        /// Reserved for local use.
        LOG_LOCAL4,
        /// Reserved for local use.
        LOG_LOCAL5,
        /// Reserved for local use.
        LOG_LOCAL6,
        /// Reserved for local use.
        LOG_LOCAL7,
    }
}
