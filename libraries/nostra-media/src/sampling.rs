#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleRange {
    pub start_frame: f64,
    pub end_frame: f64,
    pub samples: u32,
}

impl SampleRange {
    pub fn new(start_frame: f64, end_frame: f64, samples: u32) -> Self {
        Self {
            start_frame,
            end_frame,
            samples,
        }
    }

    pub fn get_samples(&self) -> Vec<f64> {
        if self.samples <= 1 {
            return vec![self.start_frame];
        }

        let mut samples = Vec::with_capacity(self.samples as usize);
        let step = (self.end_frame - self.start_frame) / (self.samples as f64 - 1.0);

        for i in 0..self.samples {
            samples.push(self.start_frame + step * (i as f64));
        }

        samples
    }
}
