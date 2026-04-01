pub mod composition;
pub mod easing;
pub mod interpolate;
pub mod keyframe;
pub mod sampling;
pub mod sources;
pub mod spring;

pub use composition::*;
pub use easing::*;
pub use interpolate::*;
pub use sampling::*;
pub use sources::*;
pub use spring::*;

#[cfg(test)]
mod tests;
