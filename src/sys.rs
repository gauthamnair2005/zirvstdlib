//! The Zirvium syscall ABI.
//!
//! Syscalls are issued with the `syscall` instruction.  The number is passed
//! in `rax`, arguments in `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9`, and the
//! result is returned in `rax`.  Negative results are negated [`Errno`]
//! values.  The kernel currently has no ring-3 userspace, but the ABI is
//! frozen here so that userland and the kernel cannot drift apart.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Syscall {
    Exit = 0,
    Write = 1,
    Read = 2,
    Open = 3,
    Close = 4,
    Mmap = 5,
    Munmap = 6,
    Yield = 7,
    GetPid = 8,
    Spawn = 9,
    Wait = 10,
    Sleep = 11,
    GetTime = 12,
    GetFramebuffer = 13,
    PollInput = 14,
    DisplayCommit = 15,
    GetNodeInfo = 16,
    Migrate = 17,
    Log = 18,
    GetGpuInfo = 19,
    Shutdown = 20,
}

impl Syscall {
    pub const fn from_u64(value: u64) -> Option<Self> {
        Some(match value {
            0 => Self::Exit,
            1 => Self::Write,
            2 => Self::Read,
            3 => Self::Open,
            4 => Self::Close,
            5 => Self::Mmap,
            6 => Self::Munmap,
            7 => Self::Yield,
            8 => Self::GetPid,
            9 => Self::Spawn,
            10 => Self::Wait,
            11 => Self::Sleep,
            12 => Self::GetTime,
            13 => Self::GetFramebuffer,
            14 => Self::PollInput,
            15 => Self::DisplayCommit,
            16 => Self::GetNodeInfo,
            17 => Self::Migrate,
            18 => Self::Log,
            19 => Self::GetGpuInfo,
            20 => Self::Shutdown,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i64)]
pub enum Errno {
    Success = 0,
    Permission = -1,
    NotFound = -2,
    Exists = -3,
    Invalid = -4,
    NoMemory = -5,
    Busy = -6,
    Io = -7,
    Unsupported = -8,
    Timeout = -9,
    Corrupt = -10,
    NoSpace = -11,
    Again = -12,
    Interrupted = -13,
}

impl Errno {
    pub const fn as_i64(self) -> i64 {
        self as i64
    }

    pub const fn from_i64(value: i64) -> Option<Self> {
        Some(match value {
            0 => Self::Success,
            -1 => Self::Permission,
            -2 => Self::NotFound,
            -3 => Self::Exists,
            -4 => Self::Invalid,
            -5 => Self::NoMemory,
            -6 => Self::Busy,
            -7 => Self::Io,
            -8 => Self::Unsupported,
            -9 => Self::Timeout,
            -10 => Self::Corrupt,
            -11 => Self::NoSpace,
            -12 => Self::Again,
            -13 => Self::Interrupted,
            _ => return None,
        })
    }

    pub const fn message(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Permission => "permission denied",
            Self::NotFound => "not found",
            Self::Exists => "already exists",
            Self::Invalid => "invalid argument",
            Self::NoMemory => "out of memory",
            Self::Busy => "device busy",
            Self::Io => "I/O error",
            Self::Unsupported => "not supported",
            Self::Timeout => "timed out",
            Self::Corrupt => "corrupt data",
            Self::NoSpace => "no space left",
            Self::Again => "try again",
            Self::Interrupted => "interrupted",
        }
    }
}

impl From<crate::error::Error> for Errno {
    fn from(error: crate::error::Error) -> Self {
        use crate::error::Error;
        match error {
            Error::OutOfMemory => Self::NoMemory,
            Error::NotFound => Self::NotFound,
            Error::AlreadyExists => Self::Exists,
            Error::InvalidArgument | Error::UnexpectedEof | Error::Overflow => Self::Invalid,
            Error::PermissionDenied => Self::Permission,
            Error::Busy => Self::Busy,
            Error::Unsupported => Self::Unsupported,
            Error::Io | Error::WriteZero => Self::Io,
            Error::Timeout => Self::Timeout,
            Error::Corrupt => Self::Corrupt,
        }
    }
}

pub type SysResult<T> = core::result::Result<T, Errno>;

/// The canonical `-1`-on-error return convention.
#[inline]
pub const fn encode_result(result: SysResult<u64>) -> i64 {
    match result {
        Ok(value) => value as i64,
        Err(error) => error.as_i64(),
    }
}

/// Decode a raw syscall return value without losing large unsigned results.
#[inline]
pub const fn decode_result(raw: i64) -> SysResult<u64> {
    if raw < 0 {
        match Errno::from_i64(raw) {
            Some(error) => Err(error),
            None => Err(Errno::Io),
        }
    } else {
        Ok(raw as u64)
    }
}
