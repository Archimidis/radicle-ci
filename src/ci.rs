use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct RadicleApiUrl(String);

impl Display for RadicleApiUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


pub type PipelineConfig = String;
pub type PipelineName = String;
pub type JobName = String;
pub type BuildName = String;

#[derive(Debug, PartialEq)]
pub enum CIResultStatus {
    Success,
    Failure,
}

#[derive(Debug)]
pub struct CIResult {
    pub status: CIResultStatus,
    pub url: String,
}

impl CIResult {
    pub fn has_completed_successfully(&self) -> bool {
        self.status == CIResultStatus::Success
    }

    pub fn get_report_message(&self) -> String {
        let status = if self.has_completed_successfully() {
            "The CI job has PASSED! 🎉"
        } else {
            "The CI job has FAILED! 🙁"
        };

        format!("{}\n\nPlease visit {} for more details.", status, self.url)
    }
}

type PatchRevisionId = String;
type PatchHead = String;
type ProjectId = String;

#[derive(Clone, Debug)]
pub struct CIJob {
    pub patch_revision_id: PatchRevisionId,
    pub patch_head: PatchHead,
    pub project_id: ProjectId,
    pub pipeline_config: PipelineConfig,
}

pub trait CI: Clone {
    fn setup(&mut self, job: CIJob) -> Result<PipelineName, anyhow::Error>;
    fn run_pipeline(&mut self, pipeline_name: &PipelineName) -> Result<CIResult, anyhow::Error>;
}
