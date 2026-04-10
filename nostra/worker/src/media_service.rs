use nostra_media::{Composition, TimelineContext};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MediaService {
    compositions: Arc<Mutex<HashMap<String, Composition>>>,
}

impl Default for MediaService {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaService {
    pub fn new() -> Self {
        Self {
            compositions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_composition(&self, composition: Composition) {
        let mut comps = self.compositions.lock().await;
        comps.insert(composition.id.clone(), composition);
    }

    pub async fn render_frame(&self, composition_id: &str, frame: f64) -> Result<Vec<u8>, String> {
        let comps = self.compositions.lock().await;
        let comp = comps
            .get(composition_id)
            .ok_or_else(|| format!("Composition {} not found", composition_id))?;

        let _ctx = TimelineContext::new(frame, comp.fps, comp.width, comp.height);
        Ok(vec![0; (comp.width * comp.height * 4) as usize])
    }

    pub async fn render_sequence(
        &self,
        composition_id: &str,
        start_frame: u32,
        end_frame: u32,
    ) -> Result<Vec<Vec<u8>>, String> {
        use rayon::prelude::*;

        let comps = self.compositions.lock().await;
        let comp = comps
            .get(composition_id)
            .ok_or_else(|| format!("Composition {} not found", composition_id))?;
        let comp_clone = comp.clone();
        drop(comps);

        let frames: Vec<u32> = (start_frame..=end_frame).collect();

        let results: Vec<Result<Vec<u8>, String>> = frames
            .into_par_iter()
            .map(|f| {
                let _ctx = TimelineContext::new(
                    f as f64,
                    comp_clone.fps,
                    comp_clone.width,
                    comp_clone.height,
                );
                Ok(vec![0; (comp_clone.width * comp_clone.height * 4) as usize])
            })
            .collect();

        results.into_iter().collect()
    }
}
