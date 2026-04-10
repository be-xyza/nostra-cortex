use crate::media_service::MediaService;
use anyhow::Result;
use std::sync::Arc;

pub async fn render_frame_activity(
    service: Arc<MediaService>,
    composition_id: String,
    frame: f64,
) -> Result<Vec<u8>> {
    println!(
        "Activity: Rendering frame {} for composition {}",
        frame, composition_id
    );

    let frame_data = service
        .render_frame(&composition_id, frame)
        .await
        .map_err(|e| anyhow::anyhow!("Render failed: {}", e))?;

    Ok(frame_data)
}

pub async fn stitch_video_activity(
    _frames_dir: String,
    _output_path: String,
    _fps: f32,
) -> Result<String> {
    println!("Activity: Stitching video to {}", _output_path);

    // In a real implementation, this would use rust-ffmpeg or call an ffmpeg subprocess.
    // We'll scaffold the logic pattern here.

    Ok(_output_path)
}
