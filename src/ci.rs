#[derive(Debug)]
pub enum CIResultStatus {
    Success,
    Failure,
}

#[derive(Debug)]
pub struct CIResult {
    pub status: CIResultStatus,
    pub message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CIJob {
    pub project_name: String,
    pub patch_branch: String,
    pub patch_head: String,
    pub project_id: String,
    pub git_uri: String,
}

pub trait CI: Clone {
    fn setup(&mut self, job: CIJob) -> Result<(), anyhow::Error>;
    fn run_pipeline(&self, project_id: &String) -> Result<CIResult, anyhow::Error>;
}
