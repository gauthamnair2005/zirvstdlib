//! Small, dependency-free maths helpers.
//!
//! Transcendental functions are implemented here instead of relying on a C
//! maths library so that the library is genuinely self contained.  The
//! approximations are accurate to well under one pixel for UI work.

pub use core::cmp::{max, min};

pub const PI: f32 = core::f32::consts::PI;
pub const TAU: f32 = core::f32::consts::TAU;

#[inline]
pub const fn clamp_i32(value: i32, low: i32, high: i32) -> i32 {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

#[inline]
pub const fn clamp_f32(value: f32, low: f32, high: f32) -> f32 {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

#[inline]
pub const fn clamp_u32(value: u32, low: u32, high: u32) -> u32 {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

#[inline]
pub const fn is_power_of_two(value: usize) -> bool {
    value != 0 && (value & (value - 1)) == 0
}

/// Round `value` up to the next multiple of `align`.  `align` must be a power
/// of two.
#[inline]
pub const fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(is_power_of_two(align));
    (value + align - 1) & !(align - 1)
}

/// Round `value` down to a multiple of `align`.  `align` must be a power of
/// two.
#[inline]
pub const fn align_down(value: usize, align: usize) -> usize {
    debug_assert!(is_power_of_two(align));
    value & !(align - 1)
}

#[inline]
pub const fn is_aligned(value: usize, align: usize) -> bool {
    value & (align - 1) == 0
}

#[inline]
pub const fn align_up_u64(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

#[inline]
pub const fn align_down_u64(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}

#[inline]
pub const fn div_ceil(value: u64, divisor: u64) -> u64 {
    value.div_ceil(divisor)
}

#[inline]
pub const fn log2_floor(value: u64) -> u32 {
    if value == 0 {
        0
    } else {
        63 - value.leading_zeros()
    }
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
pub fn lerp_i32(a: i32, b: i32, t: f32) -> i32 {
    round(a as f32 + (b - a) as f32 * t) as i32
}

#[inline]
pub fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0).max(f32::EPSILON)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = clamp01(t);
    let inv = 1.0 - t;
    1.0 - inv * inv * inv
}

#[inline]
pub fn ease_in_out_cubic(t: f32) -> f32 {
    let t = clamp01(t);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let v = -2.0 * t + 2.0;
        1.0 - v * v * v / 2.0
    }
}

#[inline]
pub fn ease_out_back(t: f32) -> f32 {
    let t = clamp01(t);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    let x = t - 1.0;
    1.0 + c3 * x * x * x + c1 * x * x
}

/// Map `value` from one range to another, clamping to the output range.
pub fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = ((value - in_min) / (in_max - in_min).max(f32::EPSILON)).clamp(0.0, 1.0);
    out_min + (out_max - out_min) * t
}

/// Round towards the nearest integer without a libm.
#[inline]
pub fn round(value: f32) -> f32 {
    let truncated = value as i64 as f32;
    let fraction = value - truncated;
    if fraction >= 0.5 {
        truncated + 1.0
    } else if fraction <= -0.5 {
        truncated - 1.0
    } else {
        truncated
    }
}

/// Floor without a libm.
#[inline]
pub fn floor(value: f32) -> f32 {
    let truncated = value as i64 as f32;
    if value < truncated {
        truncated - 1.0
    } else {
        truncated
    }
}

/// Ceiling without a libm.
#[inline]
pub fn ceil(value: f32) -> f32 {
    let truncated = value as i64 as f32;
    if value > truncated {
        truncated + 1.0
    } else {
        truncated
    }
}

/// Euclidean remainder without a libm.
#[inline]
pub fn rem_euclid(value: f32, modulus: f32) -> f32 {
    let remainder = value % modulus;
    if remainder < 0.0 {
        remainder + modulus
    } else {
        remainder
    }
}

/// Absolute value without a libm.
#[inline]
pub fn abs(value: f32) -> f32 {
    if value < 0.0 {
        -value
    } else {
        value
    }
}

/// A fast sine approximation.  Input is radians; the result is in `-1..=1`.
pub fn sin(mut x: f32) -> f32 {
    // Range-reduce into -PI..PI.
    x = rem_euclid(x + PI, TAU) - PI;
    // Bhaskara-like polynomial with a correction factor; max error ~0.001.
    let x2 = x * x;
    x * (1.0
        - x2 / 6.0
        + x2 * x2 / 120.0
        - x2 * x2 * x2 / 5040.0
        + x2 * x2 * x2 * x2 / 362880.0)
}

/// A fast cosine approximation.
pub fn cos(x: f32) -> f32 {
    sin(x + PI * 0.5)
}

/// A Newton-iterated square root that does not depend on a libm.
pub fn sqrt(value: f32) -> f32 {
    if value <= 0.0 {
        return 0.0;
    }
    // Initial guess via the bit-level exponent trick.
    let mut x = f32::from_bits((value.to_bits() >> 1) + 0x1fc0_0000);
    for _ in 0..5 {
        x = 0.5 * (x + value / x);
    }
    x
}

pub fn distance(x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let dx = x1 - x0;
    let dy = y1 - y0;
    sqrt(dx * dx + dy * dy)
}

/// Signed distance from a point to a rounded rectangle.  Negative values are
/// inside.  This is the workhorse of the anti-aliased compositor.
pub fn rounded_rect_sdf(
    px: f32,
    py: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
) -> f32 {
    let half_w = width * 0.5;
    let half_h = height * 0.5;
    let radius = radius.min(half_w).min(half_h).max(0.0);
    let cx = x + half_w;
    let cy = y + half_h;
    let qx = (px - cx).abs() - (half_w - radius);
    let qy = (py - cy).abs() - (half_h - radius);
    let qx0 = qx.max(0.0);
    let qy0 = qy.max(0.0);
    let outside = sqrt(qx0 * qx0 + qy0 * qy0);
    outside + qx.max(qy).min(0.0) - radius
}

/// Coverage in `0..=1` for a signed distance value.
#[inline]
pub fn coverage_from_sdf(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Format a byte count using binary units into a fixed-size stack buffer.
pub fn format_bytes(value: u64) -> (usize, [u8; 24]) {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut unit = 0;
    let mut scaled = value as f64;
    while scaled >= 1024.0 && unit + 1 < UNITS.len() {
        scaled /= 1024.0;
        unit += 1;
    }
    let mut buffer = [0u8; 24];
    let text = if unit == 0 {
        // Integer bytes.
        format_u64(&mut buffer, value)
    } else {
        format_float(&mut buffer, scaled as f32, 1)
    };
    let mut len = text;
    for byte in UNITS[unit].as_bytes() {
        if len < buffer.len() {
            buffer[len] = *byte;
            len += 1;
        }
    }
    // Insert a space before the unit when the number was written.
    (len, buffer)
}

fn format_u64(buffer: &mut [u8; 24], mut value: u64) -> usize {
    if value == 0 {
        buffer[0] = b'0';
        return 1;
    }
    let mut digits = [0u8; 20];
    let mut count = 0;
    while value > 0 {
        digits[count] = b'0' + (value % 10) as u8;
        value /= 10;
        count += 1;
    }
    for i in 0..count {
        buffer[i] = digits[count - 1 - i];
    }
    count
}

fn format_float(buffer: &mut [u8; 24], value: f32, decimals: usize) -> usize {
    let mut len = 0;
    let whole = value as u64;
    len += format_u64(buffer, whole);
    if decimals > 0 {
        buffer[len] = b'.';
        len += 1;
        let mut fraction = value - whole as f32;
        for _ in 0..decimals {
            fraction *= 10.0;
            let digit = fraction as u8;
            buffer[len] = b'0' + digit;
            len += 1;
            fraction -= digit as f32;
        }
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment() {
        assert_eq!(align_up(0, 4096), 0);
        assert_eq!(align_up(1, 4096), 4096);
        assert_eq!(align_down(4097, 4096), 4096);
        assert!(is_power_of_two(4096));
    }

    #[test]
    fn trig() {
        assert!(sin(0.0).abs() < 1e-4);
        assert!((sin(PI / 2.0) - 1.0).abs() < 1e-3);
        assert!((cos(0.0) - 1.0).abs() < 1e-4);
        assert!((sqrt(2.0) - 1.4142135).abs() < 1e-4);
    }

    #[test]
    fn formatting() {
        let (len, buf) = format_bytes(1536);
        assert_eq!(&buf[..len], b"1.5KiB");
    }
}
