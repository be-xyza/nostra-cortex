#[cfg(test)]
mod integration_tests {
    use crate::composition::{Sequence, TimelineContext};
    use crate::interpolate::{interpolate, ExtrapolateType, InterpolateOptions};
    use crate::spring::{spring, SpringConfig, SpringOptions};

    #[test]
    fn test_sequence_and_interpolation() {
        let seq = Sequence::new("overlay", 10, Some(50));
        let ctx = TimelineContext::new(25.0, 30.0, 1920, 1080);

        let relative_frame = seq.get_relative_frame(ctx.current_frame);
        assert_eq!(relative_frame, 15.0);

        // Animate opacity from 0 to 1 between relative frame 0 and 10
        let opacity = interpolate(
            relative_frame,
            &[0.0, 10.0],
            &[0.0, 1.0],
            Some(InterpolateOptions {
                extrapolate_left: ExtrapolateType::Clamp,
                extrapolate_right: ExtrapolateType::Clamp,
                ..Default::default()
            }),
        );
        assert_eq!(opacity, 1.0);
    }

    #[test]
    fn test_spring_in_sequence() {
        let seq = Sequence::new("popup", 0, None);
        let ctx = TimelineContext::new(5.0, 30.0, 1920, 1080);
        let rel = seq.get_relative_frame(ctx.current_frame);

        let scale = spring(
            rel,
            ctx.fps,
            Some(SpringOptions {
                from: 0.0,
                to: 1.0,
                config: SpringConfig {
                    stiffness: 100.0,
                    damping: 10.0,
                    mass: 1.0,
                    overshoot_clamping: false,
                },
                ..Default::default()
            }),
        );

        assert!(scale > 0.0);
        assert!(scale < 1.5); // Spring might overshoot
    }
}
