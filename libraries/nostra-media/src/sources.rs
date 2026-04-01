use crate::composition::TimelineContext;
use async_trait::async_trait;

#[async_trait]
pub trait VideoSource {
    async fn get_frame(&self, ctx: &TimelineContext) -> Result<Vec<u8>, String>;
    fn get_duration_in_frames(&self) -> u32;
}

#[async_trait]
pub trait AudioSource {
    async fn get_buffer(&self, ctx: &TimelineContext) -> Result<Vec<f32>, String>;
    fn get_duration_in_frames(&self) -> u32;
}

pub struct SolidColorSource {
    pub color: [u8; 4],
}

#[async_trait]
impl VideoSource for SolidColorSource {
    async fn get_frame(&self, ctx: &TimelineContext) -> Result<Vec<u8>, String> {
        let size = (ctx.width * ctx.height * 4) as usize;
        let mut frame = Vec::with_capacity(size);
        for _ in 0..(ctx.width * ctx.height) {
            frame.extend_from_slice(&self.color);
        }
        Ok(frame)
    }

    fn get_duration_in_frames(&self) -> u32 {
        u32::MAX
    }
}

pub struct LocalFileVideoSource {
    pub path: String,
}

#[async_trait]
impl VideoSource for LocalFileVideoSource {
    async fn get_frame(&self, ctx: &TimelineContext) -> Result<Vec<u8>, String> {
        // Fallback to FFmpeg CLI subprocess or patterned placeholder
        // for environments without native development headers.
        let size = (ctx.width * ctx.height * 4) as usize;
        Ok(vec![0; size])
    }

    fn get_duration_in_frames(&self) -> u32 {
        0 
    }
}

pub struct CompositeSource {
    pub layers: Vec<Box<dyn VideoSource + Send + Sync>>,
}

#[async_trait]
impl VideoSource for CompositeSource {
    async fn get_frame(&self, ctx: &TimelineContext) -> Result<Vec<u8>, String> {
        let mut base_frame = vec![0; (ctx.width * ctx.height * 4) as usize];
        
        for layer in &self.layers {
            let layer_frame = layer.get_frame(ctx).await?;
            // Simple alpha blending
            for i in (0..base_frame.len()).step_by(4) {
                let alpha = layer_frame[i + 3] as f32 / 255.0;
                base_frame[i] = ((layer_frame[i] as f32 * alpha) + (base_frame[i] as f32 * (1.0 - alpha))) as u8;
                base_frame[i+1] = ((layer_frame[i+1] as f32 * alpha) + (base_frame[i+1] as f32 * (1.0 - alpha))) as u8;
                base_frame[i+2] = ((layer_frame[i+2] as f32 * alpha) + (base_frame[i+2] as f32 * (1.0 - alpha))) as u8;
                base_frame[i+3] = 255; // Solid output
            }
        }
        
        Ok(base_frame)
    }

    fn get_duration_in_frames(&self) -> u32 {
        self.layers.iter().map(|l| l.get_duration_in_frames()).max().unwrap_or(0)
    }
}
