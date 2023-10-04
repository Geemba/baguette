//! # baguette-math
//!  a tiny wrapper around baguette's math library of choice glam and other utility crates

pub use glam as math;
pub use fastrand as rand;

pub type Vec3 = glam::Vec3; pub type IVec3 = glam::IVec3;
pub type Vec2 = glam::Vec2; pub type IVec2 = glam::IVec2;