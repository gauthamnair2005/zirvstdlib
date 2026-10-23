//! The Zirvium prelude.
//!
//! `use zirvstdlib::prelude::*;` brings the common types and traits into
//! scope without pulling in a hosted standard library.

pub use crate::collections::{Arc, BTreeMap, BTreeSet, String, ToString, Vec};
pub use crate::color::{Color, Rgba, blend_argb};
pub use crate::geometry::{Edges, Point, Rect, Size};
pub use crate::math::{clamp_f32, clamp_i32, clamp01, lerp, smoothstep};
pub use crate::sync::{Once, SpinLock};
pub use crate::{Error, Result, format, vec};
pub use alloc::boxed::Box;
pub use core::fmt::Write as FmtWrite;
