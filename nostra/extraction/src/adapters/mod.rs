use anyhow::Result;

use crate::{ExtractionRequestV1, ExtractionResultV1, PipelineAdapter, run_local_pipeline};

#[derive(Default)]
pub struct LocalPipelineAdapter;

impl PipelineAdapter for LocalPipelineAdapter {
    fn adapter_id(&self) -> &'static str {
        "local_pipeline"
    }

    fn run(&self, request: &ExtractionRequestV1) -> Result<ExtractionResultV1> {
        run_local_pipeline(request)
    }
}
