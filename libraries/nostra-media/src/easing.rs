pub type EasingFunction = fn(f64) -> f64;

pub fn linear(t: f64) -> f64 {
    t
}

pub mod easing {
    use super::EasingFunction;

    pub const LINEAR: EasingFunction = |t| t;

    pub fn ease_in(t: f64) -> f64 {
        t * t
    }

    pub fn ease_out(t: f64) -> f64 {
        t * (2.0 - t)
    }

    pub fn ease_in_out(t: f64) -> f64 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }
}
