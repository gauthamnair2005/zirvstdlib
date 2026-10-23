//! Logging interface shared by the kernel and drivers.

use crate::fmt::FixedString;
use crate::sync::SpinLock;
use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl Level {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }

    pub const fn color(self) -> u32 {
        match self {
            Self::Error => 0xffef_4444,
            Self::Warn => 0xfff5_9e0b,
            Self::Info => 0xff38_bdf8,
            Self::Debug => 0xffa3_a3a3,
            Self::Trace => 0xff6b_7280,
        }
    }
}

/// A log record.  The message lives on the stack for the duration of
/// [`Logger::log`].
pub struct Record<'a> {
    pub level: Level,
    pub module: &'a str,
    pub message: &'a str,
    pub timestamp_ms: u64,
}

pub trait Logger: Sync {
    fn log(&self, record: &Record<'_>);
}

struct NullLogger;

impl Logger for NullLogger {
    fn log(&self, _record: &Record<'_>) {}
}

static LOGGER: SpinLock<Option<&'static dyn Logger>> = SpinLock::new(None);
static NULL_LOGGER: NullLogger = NullLogger;

/// Install the global logger.  May be called once; later calls are ignored.
pub fn set_logger(logger: &'static dyn Logger) {
    let mut slot = LOGGER.lock();
    if slot.is_none() {
        *slot = Some(logger);
    }
}

/// Emit a log record.
pub fn log(level: Level, module: &str, arguments: fmt::Arguments<'_>) {
    let mut message = FixedString::<256>::new();
    if fmt::write(&mut message, arguments).is_err() {
        return;
    }
    let record = Record {
        level,
        module,
        message: message.as_str(),
        timestamp_ms: crate::time::monotonic_ms(),
    };
    let logger = {
        let slot = LOGGER.lock();
        match *slot {
            Some(logger) => logger,
            None => &NULL_LOGGER,
        }
    };
    logger.log(&record);
}

/// Emit an informational message.
pub fn info(module: &str, arguments: fmt::Arguments<'_>) {
    log(Level::Info, module, arguments);
}

/// Emit a warning.
pub fn warn(module: &str, arguments: fmt::Arguments<'_>) {
    log(Level::Warn, module, arguments);
}

/// Emit an error.
pub fn error(module: &str, arguments: fmt::Arguments<'_>) {
    log(Level::Error, module, arguments);
}

/// Emit a debug message.
pub fn debug(module: &str, arguments: fmt::Arguments<'_>) {
    log(Level::Debug, module, arguments);
}

#[macro_export]
macro_rules! zlog {
    ($level:expr, $module:expr, $($arg:tt)*) => {
        $crate::log::log($level, $module, format_args!($($arg)*))
    };
}
