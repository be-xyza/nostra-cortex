use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub duration_in_frames: u32,
}

impl Composition {
    pub fn new(id: &str, width: u32, height: u32, fps: f32, duration_in_frames: u32) -> Self {
        Self {
            id: id.to_string(),
            width,
            height,
            fps,
            duration_in_frames,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence {
    pub name: String,
    pub from: i32,
    pub duration: Option<u32>,
}

impl Sequence {
    pub fn new(name: &str, from: i32, duration: Option<u32>) -> Self {
        Self {
            name: name.to_string(),
            from,
            duration,
        }
    }

    pub fn get_relative_frame(&self, absolute_frame: f64) -> f64 {
        absolute_frame - self.from as f64
    }
}

#[derive(Debug, Clone)]
pub struct TimelineContext {
    pub current_frame: f64,
    pub fps: f32,
    pub width: u32,
    pub height: u32,
}

impl TimelineContext {
    pub fn new(current_frame: f64, fps: f32, width: u32, height: u32) -> Self {
        Self {
            current_frame,
            fps,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub sequences: Vec<Sequence>,
}

impl Series {
    pub fn new(sequences: Vec<Sequence>) -> Self {
        Self { sequences }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loop {
    pub sequence: Sequence,
    pub times: u32,
}

impl Loop {
    pub fn new(sequence: Sequence, times: u32) -> Self {
        Self { sequence, times }
    }
}
