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
    pub fn has_completed_successfully(self: &Self) -> bool {
        self.status == CIResultStatus::Success
    }

    pub fn get_report_message(self: &Self) -> String {
        let status = if self.has_completed_successfully() {
            "The CI job has PASSED! 🎉"
        } else {
            "The CI job has FAILED! 🙁"
        };

        format!("{}\n\nPlease visit {} for more details.", status, self.url)
    }
}

#[derive(Clone, Debug)]
pub struct CIJob {
    pub patch_revision_id: String,
    pub patch_head: String,
    pub project_id: String,
    pub git_uri: String,
}

pub trait CI: Clone {
    fn setup(&mut self, job: CIJob) -> Result<PipelineName, anyhow::Error>;
    fn run_pipeline(&self, pipeline_name: &PipelineName) -> Result<CIResult, anyhow::Error>;
}
