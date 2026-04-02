use crate::interpolate::interpolate;

#[derive(Debug, Clone, Copy)]
pub struct SpringConfig {
    pub damping: f64,
    pub mass: f64,
    pub stiffness: f64,
    pub overshoot_clamping: bool,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            damping: 10.0,
            mass: 1.0,
            stiffness: 100.0,
            overshoot_clamping: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnimationNode {
    pub last_timestamp: f64,
    pub to_value: f64,
    pub current: f64,
    pub velocity: f64,
    pub prev_position: f64,
}

fn advance(animation: AnimationNode, now: f64, config: &SpringConfig) -> AnimationNode {
    let AnimationNode {
        to_value,
        last_timestamp,
        current,
        velocity,
        ..
    } = animation;

    let delta_time = (now - last_timestamp).min(64.0);

    assert!(
        config.damping > 0.0,
        "Spring damping must be greater than 0"
    );

    let c = config.damping;
    let m = config.mass;
    let k = config.stiffness;

    let v0 = -velocity;
    let x0 = to_value - current;

    let zeta = c / (2.0 * (k * m).sqrt()); // damping ratio
    let omega0 = (k / m).sqrt(); // undamped angular frequency
    let omega1 = omega0 * (1.0 - zeta.powi(2)).sqrt(); // exponential decay

    let t = delta_time / 1000.0;

    let sin1 = (omega1 * t).sin();
    let cos1 = (omega1 * t).cos();

    // under damped
    let under_damped_envelope = (-zeta * omega0 * t).exp();
    let under_damped_frag1 =
        under_damped_envelope * (sin1 * ((v0 + zeta * omega0 * x0) / omega1) + x0 * cos1);

    let under_damped_position = to_value - under_damped_frag1;
    let under_damped_velocity = zeta * omega0 * under_damped_frag1
        - under_damped_envelope * (cos1 * (v0 + zeta * omega0 * x0) - omega1 * x0 * sin1);

    // critically damped
    let critically_damped_envelope = (-omega0 * t).exp();
    let critically_damped_position =
        to_value - critically_damped_envelope * (x0 + (v0 + omega0 * x0) * t);

    let critically_damped_velocity =
        critically_damped_envelope * (v0 * (t * omega0 - 1.0) + t * x0 * omega0 * omega0);

    AnimationNode {
        to_value,
        prev_position: current,
        last_timestamp: now,
        current: if zeta < 1.0 {
            under_damped_position
        } else {
            critically_damped_position
        },
        velocity: if zeta < 1.0 {
            under_damped_velocity
        } else {
            critically_damped_velocity
        },
    }
}

pub fn spring_calculation(frame: f64, fps: f32, config: &SpringConfig) -> AnimationNode {
    let from = 0.0;
    let to = 1.0;

    let mut animation = AnimationNode {
        last_timestamp: 0.0,
        current: from,
        to_value: to,
        velocity: 0.0,
        prev_position: 0.0,
    };

    let frame_clamped = frame.max(0.0);
    let uneven_rest = frame_clamped % 1.0;
    let floor_frame = frame_clamped.floor() as i32;

    for f_idx in 0..=floor_frame {
        let mut f = f_idx as f64;
        if f_idx == floor_frame {
            f += uneven_rest;
        }

        let time = (f / fps as f64) * 1000.0;
        animation = advance(animation, time, config);
    }

    animation
}

pub struct SpringOptions {
    pub config: SpringConfig,
    pub from: f64,
    pub to: f64,
    pub duration_in_frames: Option<u32>,
    pub duration_rest_threshold: Option<f64>,
    pub delay: u32,
    pub reverse: bool,
}

impl Default for SpringOptions {
    fn default() -> Self {
        Self {
            config: SpringConfig::default(),
            from: 0.0,
            to: 1.0,
            duration_in_frames: None,
            duration_rest_threshold: None,
            delay: 0,
            reverse: false,
        }
    }
}

pub fn spring(frame: f64, fps: f32, options: Option<SpringOptions>) -> f64 {
    let options = options.unwrap_or_default();
    let SpringOptions {
        config,
        from,
        to,
        duration_in_frames,
        duration_rest_threshold,
        delay,
        reverse,
    } = options;

    let natural_duration = if reverse || duration_in_frames.is_some() {
        Some(measure_spring(
            fps,
            &config,
            duration_rest_threshold.unwrap_or(0.005),
        ))
    } else {
        None
    };

    let reverse_processed = if reverse {
        (duration_in_frames
            .map(|d| d as f64)
            .unwrap_or(natural_duration.unwrap() as f64))
            - frame
    } else {
        frame
    };

    let delay_processed = reverse_processed
        + if reverse {
            delay as f64
        } else {
            -(delay as f64)
        };

    let duration_processed = match duration_in_frames {
        None => delay_processed,
        Some(d) => {
            if delay_processed > d as f64 {
                return to;
            }
            delay_processed / (d as f64 / natural_duration.unwrap() as f64)
        }
    };

    let spr = spring_calculation(duration_processed, fps, &config);

    let inner = if config.overshoot_clamping {
        if to >= from {
            spr.current.min(to)
        } else {
            spr.current.max(to)
        }
    } else {
        spr.current
    };

    if from == 0.0 && to == 1.0 {
        inner
    } else {
        interpolate(inner, &[0.0, 1.0], &[from, to], None)
    }
}

pub fn measure_spring(fps: f32, config: &SpringConfig, threshold: f64) -> u32 {
    if threshold == 0.0 {
        return 1000;
    } // Avoid infinite loop
    if threshold >= 1.0 {
        return 0;
    }

    let mut frame = 0;

    let calc = |f: f64| spring_calculation(f, fps, config);

    let mut animation = calc(frame as f64);
    let mut difference = (animation.current - animation.to_value).abs();

    while difference >= threshold && frame < 10000 {
        frame += 1;
        animation = calc(frame as f64);
        difference = (animation.current - animation.to_value).abs();
    }

    let mut finished_frame = frame;
    let mut i = 0;
    while i < 20 && frame < 10000 {
        frame += 1;
        animation = calc(frame as f64);
        difference = (animation.current - animation.to_value).abs();
        if difference >= threshold {
            i = 0;
            finished_frame = frame + 1;
        } else {
            i += 1;
        }
    }

    finished_frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_basic() {
        let val = spring(0.0, 30.0, None);
        assert_eq!(val, 0.0);
        let val_end = spring(100.0, 30.0, None);
        assert!(val_end > 0.99);
    }
}
