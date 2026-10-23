//! The Zirvium prelude.
//!
//! `use zirvstdlib::prelude::*;` brings the common types and traits into
//! scope without pulling in a hosted standard library.

pub use crate::collections::{Arc, BTreeMap, BTreeSet, String, ToString, Vec};
pub use crate::color::{blend_argb, Color, Rgba};
pub use crate::geometry::{Edges, Point, Rect, Size};
pub use crate::math::{clamp01, clamp_f32, clamp_i32, lerp, smoothstep};
pub use crate::sync::{Once, SpinLock};
pub use crate::{format, vec, Error, Result};
pub use alloc::boxed::Box;
pub use core::fmt::Write as FmtWrite;
