use crate::interpolate::{interpolate, ExtrapolateType, InterpolateOptions};

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub frame: f64,
    pub value: f64,
}

pub struct KeyframeSequence {
    pub keyframes: Vec<Keyframe>,
}

impl KeyframeSequence {
    pub fn new(mut keyframes: Vec<Keyframe>) -> Self {
        keyframes.sort_by(|a, b| a.frame.partial_cmp(&b.frame).unwrap());
        Self { keyframes }
    }

    pub fn evaluate(&self, frame: f64) -> f64 {
        if self.keyframes.is_empty() {
            return 0.0;
        }

        let input_range: Vec<f64> = self.keyframes.iter().map(|k| k.frame).collect();
        let output_range: Vec<f64> = self.keyframes.iter().map(|k| k.value).collect();

        interpolate(
            frame,
            &input_range,
            &output_range,
            Some(InterpolateOptions {
                extrapolate_left: ExtrapolateType::Clamp,
                extrapolate_right: ExtrapolateType::Clamp,
                ..Default::default()
            }),
        )
    }
}
