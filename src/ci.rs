use std::fmt::{Display, Formatter};

use serde::Deserialize;

#[derive(Clone, PartialEq)]
pub struct ConcourseUrl(pub String);

impl Display for ConcourseUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq)]
pub struct CIConfig {
    pub concourse_url: ConcourseUrl,
    pub ci_user: String,
    pub ci_pass: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadicleApiUrl(pub String);

impl Display for RadicleApiUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineConfig(pub String);

impl PipelineConfig {
    pub fn replace(&self, old: &str, new: &str) -> PipelineConfig {
        PipelineConfig(self.0.replace(old, new))
    }
}

impl Display for PipelineConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineName(pub String);

impl Display for PipelineName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct JobName(pub String);

impl Display for JobName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct BuildName(pub String);

impl Display for BuildName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

pub trait CIObserver: PartialEq {
    fn update(&self, build: &CIResult);
}

pub trait CIObservable<'a, T> where T: CIObserver {
    fn attach(&mut self, observer: &'a T);
    fn detach(&mut self, observer: &'a T);
    fn notify(&self, build: &CIResult);
}

pub trait CI {
    fn setup(&mut self, job: CIJob) -> Result<PipelineName, anyhow::Error>;
    fn run_pipeline(&mut self, pipeline_name: &PipelineName) -> Result<CIResult, anyhow::Error>;
}

