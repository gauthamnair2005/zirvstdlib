//! Formatting helpers used by the kernel console, the shell and the UI.

use core::fmt;

pub use core::fmt::Write;

/// A display wrapper that writes a byte slice as lowercase hexadecimal.
pub struct Hex<'a>(pub &'a [u8]);

impl fmt::Display for Hex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// A display wrapper that writes a number as `0x`-prefixed hexadecimal.
pub struct HexU64(pub u64);

impl fmt::Display for HexU64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

/// A display wrapper for a byte count with binary units.
pub struct ByteSize(pub u64);

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
        let mut value = self.0 as f64;
        let mut unit = 0;
        while value >= 1024.0 && unit + 1 < UNITS.len() {
            value /= 1024.0;
            unit += 1;
        }
        if unit == 0 {
            write!(f, "{} {}", self.0, UNITS[unit])
        } else {
            write!(f, "{value:.1} {}", UNITS[unit])
        }
    }
}

/// A stack-allocated string with a fixed capacity, useful in interrupt
/// handlers where allocating is forbidden.
pub struct FixedString<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> FixedString<N> {
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: only `fmt::Write::write_str` writes into the buffer and it
        // only copies complete UTF-8 strings.
        unsafe { core::str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> Default for FixedString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> fmt::Write for FixedString<N> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let available = N.saturating_sub(self.len);
        if text.len() > available {
            return Err(fmt::Error);
        }
        self.bytes[self.len..self.len + text.len()].copy_from_slice(text.as_bytes());
        self.len += text.len();
        Ok(())
    }
}

impl<const N: usize> fmt::Display for FixedString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Write a string into a fixed buffer, returning the used prefix.
pub fn write_str_into<'a>(buffer: &'a mut [u8], text: &str) -> &'a str {
    let count = buffer.len().min(text.len());
    buffer[..count].copy_from_slice(&text.as_bytes()[..count]);
    // SAFETY: truncating at a byte boundary could split UTF-8.  Callers pass
    // ASCII for status text; fall back to an empty string if invalid.
    core::str::from_utf8(&buffer[..count]).unwrap_or("")
}

/// Zero-pad a number to `width` digits.
pub fn write_padded(
    f: &mut fmt::Formatter<'_>,
    value: u64,
    width: usize,
    fill: char,
) -> fmt::Result {
    let mut buffer = [0u8; 20];
    let mut index = buffer.len();
    let mut remaining = value;
    loop {
        index -= 1;
        buffer[index] = b'0' + (remaining % 10) as u8;
        remaining /= 10;
        if remaining == 0 {
            break;
        }
    }
    let digits = &buffer[index..];
    for _ in digits.len()..width {
        write!(f, "{fill}")?;
    }
    for byte in digits {
        write!(f, "{}", *byte as char)?;
    }
    Ok(())
}

/// Format an uptime given in milliseconds as `1d 02:03:04`.
pub fn format_uptime(milliseconds: u64) -> FixedString<32> {
    use fmt::Write;
    let total_seconds = milliseconds / 1000;
    let days = total_seconds / 86_400;
    let hours = (total_seconds / 3_600) % 24;
    let minutes = (total_seconds / 60) % 60;
    let seconds = total_seconds % 60;
    let mut output = FixedString::new();
    if days > 0 {
        let _ = write!(output, "{days}d ");
    }
    let _ = write!(output, "{hours:02}:{minutes:02}:{seconds:02}");
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;

    #[test]
    fn fixed_string() {
        let mut text = FixedString::<16>::new();
        write!(text, "hello {}", 42).unwrap();
        assert_eq!(text.as_str(), "hello 42");
    }

    #[test]
    fn byte_sizes() {
        assert_eq!(alloc::format!("{}", ByteSize(512)), "512 B");
        assert_eq!(alloc::format!("{}", ByteSize(1536)), "1.5 KiB");
    }
}
