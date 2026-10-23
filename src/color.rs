//! Colours, packing and alpha compositing.
//!
//! DisplayJet stores pixels as `0xAARRGGBB` in a `u32`.  This matches the
//! little-endian byte order `B,G,R,A` used by virtually every modern
//! framebuffer, so the buffer can be handed to a blitter or a GPU resource
//! without conversion.

use core::fmt;

/// An 8-bit-per-channel RGBA colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub type Color = Rgba;

impl Rgba {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const RED: Self = Self::new(239, 68, 68, 255);
    pub const GREEN: Self = Self::new(34, 197, 94, 255);
    pub const BLUE: Self = Self::new(59, 130, 246, 255);
    pub const YELLOW: Self = Self::new(250, 204, 21, 255);
    pub const CYAN: Self = Self::new(34, 211, 238, 255);
    pub const MAGENTA: Self = Self::new(217, 70, 239, 255);
    pub const ORANGE: Self = Self::new(249, 115, 22, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Build a colour from `0xRRGGBB`.
    pub const fn from_rgb_hex(value: u32) -> Self {
        Self::rgb(
            ((value >> 16) & 0xff) as u8,
            ((value >> 8) & 0xff) as u8,
            (value & 0xff) as u8,
        )
    }

    /// Build a colour from `0xAARRGGBB`.
    pub const fn from_argb(value: u32) -> Self {
        Self::new(
            ((value >> 16) & 0xff) as u8,
            ((value >> 8) & 0xff) as u8,
            (value & 0xff) as u8,
            ((value >> 24) & 0xff) as u8,
        )
    }

    /// Pack into `0xAARRGGBB`.
    pub const fn to_argb(self) -> u32 {
        ((self.a as u32) << 24)
            | ((self.r as u32) << 16)
            | ((self.g as u32) << 8)
            | (self.b as u32)
    }

    /// Pack into `0xRRGGBB`, discarding alpha.
    pub const fn to_rgb(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Return a copy with a different alpha value.
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Multiply the alpha by `factor` (0..=255).
    pub const fn scale_alpha(self, factor: u8) -> Self {
        let a = ((self.a as u32 * factor as u32 + 127) / 255) as u8;
        self.with_alpha(a)
    }

    /// Linear interpolation in sRGB space.  `t` is clamped to `0..=1`.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let mix = |a: u8, b: u8| -> u8 {
            crate::math::round(a as f32 + (b as f32 - a as f32) * t)
                .clamp(0.0, 255.0) as u8
        };
        Self::new(
            mix(self.r, other.r),
            mix(self.g, other.g),
            mix(self.b, other.b),
            mix(self.a, other.a),
        )
    }

    /// Convert from HSV.  Hue is in degrees, saturation/value in `0..=1`.
    pub fn from_hsv(hue: f32, saturation: f32, value: f32) -> Self {
        let h = crate::math::rem_euclid(hue, 360.0) / 60.0;
        let s = saturation.clamp(0.0, 1.0);
        let v = value.clamp(0.0, 1.0);
        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let to_u8 = |f: f32| crate::math::round((f + m) * 255.0).clamp(0.0, 255.0) as u8;
        Self::new(to_u8(r), to_u8(g), to_u8(b), 255)
    }

    /// Convert to HSV `(hue, saturation, value)`.
    pub fn to_hsv(self) -> (f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let hue = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };
        let saturation = if max == 0.0 { 0.0 } else { delta / max };
        (crate::math::rem_euclid(hue, 360.0), saturation, max)
    }

    /// Alpha-blend `self` over `dst`.
    #[inline]
    pub fn blend_over(self, dst: Self) -> Self {
        if self.a == 255 {
            return self;
        }
        if self.a == 0 {
            return dst;
        }
        let a = self.a as u32;
        let inv = 255 - a;
        let alpha = self.a as u32 + (dst.a as u32 * inv + 127) / 255;
        Self::new(
            blend_channel(self.r, dst.r, a, inv),
            blend_channel(self.g, dst.g, a, inv),
            blend_channel(self.b, dst.b, a, inv),
            if alpha > 255 { 255 } else { alpha as u8 },
        )
    }
}

#[inline]
fn blend_channel(src: u8, dst: u8, alpha: u32, inverse: u32) -> u8 {
    ((src as u32 * alpha + dst as u32 * inverse + 127) / 255) as u8
}

/// Alpha-blend one packed ARGB pixel over another.
#[inline]
pub const fn blend_argb(src: u32, dst: u32) -> u32 {
    let a = src >> 24;
    if a == 0 {
        return dst;
    }
    if a == 255 {
        return src;
    }
    let inv = 255 - a;
    let rb = (((src & 0x00ff00ff) * a + (dst & 0x00ff00ff) * inv + 0x00800080) >> 8)
        & 0x00ff00ff;
    let g = (((src & 0x0000ff00) * a + (dst & 0x0000ff00) * inv + 0x00008000) >> 8)
        & 0x0000ff00;
    rb | g
}

/// A vertical or horizontal linear gradient with multiple stops.
#[derive(Clone, Copy, Debug)]
pub struct GradientStop {
    /// Position in `0..=1`.
    pub position: f32,
    pub color: Rgba,
}

impl GradientStop {
    pub const fn new(position: f32, color: Rgba) -> Self {
        Self { position, color }
    }
}

/// Sample a sorted gradient at `t`.
pub fn sample_gradient(stops: &[GradientStop], t: f32) -> Rgba {
    if stops.is_empty() {
        return Rgba::TRANSPARENT;
    }
    let t = t.clamp(0.0, 1.0);
    if t <= stops[0].position {
        return stops[0].color;
    }
    for window in stops.windows(2) {
        let (a, b) = (window[0], window[1]);
        if t <= b.position {
            let span = (b.position - a.position).max(f32::EPSILON);
            return a.color.lerp(b.color, (t - a.position) / span);
        }
    }
    stops[stops.len() - 1].color
}

impl fmt::Display for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packing_round_trip() {
        let c = Rgba::new(12, 34, 56, 78);
        assert_eq!(c.to_argb(), 0x4e0c2238);
        assert_eq!(Rgba::from_argb(c.to_argb()), c);
    }

    #[test]
    fn blending() {
        assert_eq!(blend_argb(0xffffffff, 0xff000000), 0xffffffff);
        assert_eq!(blend_argb(0x00ffffff, 0xff123456), 0xff123456);
        let mid = Rgba::new(255, 255, 255, 128).blend_over(Rgba::BLACK);
        assert!(mid.r > 120 && mid.r < 136);
    }

    #[test]
    fn hsv_round_trip() {
        let c = Rgba::from_hsv(210.0, 0.8, 0.9);
        let (h, s, v) = c.to_hsv();
        assert!((h - 210.0).abs() < 0.5);
        assert!((s - 0.8).abs() < 0.02);
        assert!((v - 0.9).abs() < 0.02);
    }
}
