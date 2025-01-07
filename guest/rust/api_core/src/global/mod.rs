mod state;
pub use state::*;

mod runtime;
pub use runtime::*;

mod entity_id;
pub use entity_id::*;

mod shapes;
pub use shapes::*;

mod procedurals;
pub use procedurals::*;

// Re-exports from other crates.
pub use ambient_shared_types::{CursorIcon, ModifiersState, MouseButton, VirtualKeyCode};
pub use futures::{Future, FutureExt};
pub use glam;
pub use glam::{f32::*, i32::*, u32::*, Vec2Swizzles, Vec3Swizzles, Vec4Swizzles};
pub use serde;
pub use std::time::Duration;
pub use ulid::Ulid;

/// In Rust, functions that can fail are expected to return a [Result] type.
/// [ResultEmpty] is a [Result] type that has no value and can accept
/// any kind of error through the question-mark operator `?`.
///
/// It is used as the default return type for Ambient operations that take
/// a callback.
pub type ResultEmpty = anyhow::Result<()>;

/// The default "happy path" value for an [ResultEmpty]. You can return this
/// from a handler to signal that there are no issues.
#[allow(non_upper_case_globals)]
pub const OkEmpty: ResultEmpty = Ok(());

#[inline]
/// Helper function that returns the [Default](std::default::Default::default) for the type `T`.
/// Most useful with struct update syntax, or with initializing components.
pub fn default<T: Default>() -> T {
    std::default::Default::default()
}

#[inline]
/// Lerp (linear-interpolate) between any two values that support addition and multiplication.
///
/// Note that there may be better domain-specific ways to lerp between values, especially
/// for quaternions and colors.
pub fn lerp<T: std::ops::Add + std::ops::Mul<f32>>(
    a: T,
    b: T,
    t: f32,
) -> <<T as std::ops::Mul<f32>>::Output as std::ops::Add>::Output
where
    <T as std::ops::Mul<f32>>::Output: std::ops::Add,
{
    a * (1.0 - t) + b * t
}
