//! The Zirvium error type.

use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    OutOfMemory,
    NotFound,
    AlreadyExists,
    InvalidArgument,
    PermissionDenied,
    Busy,
    Unsupported,
    Io,
    UnexpectedEof,
    WriteZero,
    Overflow,
    Timeout,
    Corrupt,
}

impl Error {
    /// A short, stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OutOfMemory => "out of memory",
            Self::NotFound => "not found",
            Self::AlreadyExists => "already exists",
            Self::InvalidArgument => "invalid argument",
            Self::PermissionDenied => "permission denied",
            Self::Busy => "busy",
            Self::Unsupported => "unsupported",
            Self::Io => "I/O error",
            Self::UnexpectedEof => "unexpected end of file",
            Self::WriteZero => "write returned zero",
            Self::Overflow => "arithmetic overflow",
            Self::Timeout => "timed out",
            Self::Corrupt => "corrupt data",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<core::fmt::Error> for Error {
    fn from(_: core::fmt::Error) -> Self {
        Self::InvalidArgument
    }
}

pub type Result<T> = core::result::Result<T, Error>;
