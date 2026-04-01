use crate::easing::EasingFunction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtrapolateType {
    Extend,
    Identity,
    Clamp,
    Wrap,
}

pub struct InterpolateOptions {
    pub easing: EasingFunction,
    pub extrapolate_left: ExtrapolateType,
    pub extrapolate_right: ExtrapolateType,
}

impl Default for InterpolateOptions {
    fn default() -> Self {
        Self {
            easing: crate::easing::linear,
            extrapolate_left: ExtrapolateType::Extend,
            extrapolate_right: ExtrapolateType::Extend,
        }
    }
}

fn interpolate_function(
    input: f64,
    input_range: [f64; 2],
    output_range: [f64; 2],
    options: &InterpolateOptions,
) -> f64 {
    let InterpolateOptions {
        extrapolate_left,
        extrapolate_right,
        easing,
    } = options;

    let mut result = input;
    let [input_min, input_max] = input_range;
    let [output_min, output_max] = output_range;

    if result < input_min {
        match extrapolate_left {
            ExtrapolateType::Identity => return result,
            ExtrapolateType::Clamp => result = input_min,
            ExtrapolateType::Wrap => {
                let range = input_max - input_min;
                result = ((((result - input_min) % range) + range) % range) + input_min;
            }
            ExtrapolateType::Extend => {}
        }
    }

    if result > input_max {
        match extrapolate_right {
            ExtrapolateType::Identity => return result,
            ExtrapolateType::Clamp => result = input_max,
            ExtrapolateType::Wrap => {
                let range = input_max - input_min;
                result = ((((result - input_min) % range) + range) % range) + input_min;
            }
            ExtrapolateType::Extend => {}
        }
    }

    if (output_min - output_max).abs() < f64::EPSILON {
        return output_min;
    }

    // Input Range
    result = (result - input_min) / (input_max - input_min);

    // Easing
    result = easing(result);

    // Output Range
    result = result * (output_max - output_min) + output_min;

    result
}

fn find_range(input: f64, input_range: &[f64]) -> usize {
    let mut i = 1;
    while i < input_range.len() - 1 {
        if input_range[i] >= input {
            break;
        }
        i += 1;
    }
    i - 1
}

pub fn interpolate(
    input: f64,
    input_range: &[f64],
    output_range: &[f64],
    options: Option<InterpolateOptions>,
) -> f64 {
    assert_eq!(
        input_range.len(),
        output_range.len(),
        "inputRange and outputRange must have the same length"
    );
    assert!(
        input_range.len() >= 2,
        "inputRange must have at least 2 elements"
    );

    // Validation: Strictly monotonically increasing
    for i in 1..input_range.len() {
        assert!(
            input_range[i] > input_range[i - 1],
            "inputRange must be strictly monotonically increasing"
        );
    }

    let options = options.unwrap_or_default();
    let range = find_range(input, input_range);

    interpolate_function(
        input,
        [input_range[range], input_range[range + 1]],
        [output_range[range], output_range[range + 1]],
        &options,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_interpolation() {
        let input_range = [0.0, 1.0];
        let output_range = [0.0, 100.0];
        assert_eq!(interpolate(0.5, &input_range, &output_range, None), 50.0);
    }

    #[test]
    fn test_clamping() {
        let input_range = [0.0, 1.0];
        let output_range = [0.0, 100.0];
        let options = InterpolateOptions {
            extrapolate_left: ExtrapolateType::Clamp,
            extrapolate_right: ExtrapolateType::Clamp,
            ..Default::default()
        };
        assert_eq!(
            interpolate(-1.0, &input_range, &output_range, Some(options)),
            0.0
        );
    }
}
