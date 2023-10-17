use std::time::Duration;

use anyhow::anyhow;
use radicle_term as term;
use tokio::time::sleep;

use crate::ci::{CI, CIJob, CIResult, CIResultStatus, ConcourseUrl, PipelineConfig, PipelineName, RadicleApiUrl};
use crate::concourse::api::ConcourseAPI;
use crate::concourse::build::{Build, BuildID};

pub struct ConcourseCI {
    runtime: tokio::runtime::Runtime,
    api: ConcourseAPI,
    radicle_api_url: RadicleApiUrl,
    concourse_url: ConcourseUrl,
}

impl Clone for ConcourseCI {
    fn clone(&self) -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
            api: self.api.clone(),
            radicle_api_url: self.radicle_api_url.clone(),
            concourse_url: self.concourse_url.clone(),
        }
    }
}

impl ConcourseCI {
    pub fn new(radicle_api_url: RadicleApiUrl, concourse_url: ConcourseUrl, ci_user: String, ci_pass: String) -> ConcourseCI {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let api = ConcourseAPI::new(concourse_url.clone(), ci_user, ci_pass);

        Self { runtime, api, concourse_url, radicle_api_url }
    }

    pub async fn watch_pipeline_job_build(&mut self, build_id: BuildID) -> Result<Build, anyhow::Error> {
        loop {
            sleep(Duration::from_secs(3)).await;

            let build_result = self.api.get_build(&build_id).await;
            match build_result {
                Ok(build) => {
                    if build.has_completed() {
                        term::info!("Pipeline job build #{} has completed execution", build_id);
                        break Ok(build);
                    }
                }
                Err(error) => {
                    term::info!("Failed to get pipeline job build {:#?}", error);
                    break Err(anyhow::anyhow!("Failed to get pipeline job build"));
                }
            }
        }
    }
}

fn create_concourse_pipeline_config(radicle_api_url: &RadicleApiUrl, job: &CIJob) -> PipelineConfig {
    let repo_url = format!("{}/{}.git", radicle_api_url, job.project_id);

    job.pipeline_config
        .replace("((repo_url))", repo_url.as_str())
        .replace("((patch_revision_id))", job.patch_revision_id.as_str())
        .replace("((patch_head))", job.patch_head.as_str())
}

impl CI for ConcourseCI {
    fn setup(&mut self, job: CIJob) -> Result<PipelineName, anyhow::Error> {
        self.runtime.block_on(async {
            let concourse_config = create_concourse_pipeline_config(&self.radicle_api_url, &job);
            let pipeline_name = PipelineName(format!("{}-pipeline", job.project_id));

            let result = self.api.get_access_token().await;
            if result.is_err() {
                return Err(anyhow::anyhow!("Failed to get access token"));
            }

            let result = self.api.get_pipeline_config(&pipeline_name).await;
            let config_version = match result {
                Ok(config) => config.version,
                Err(_) => None,
            };

            term::info!("Triggering pipeline {} creation with current version {:?}", pipeline_name, config_version);
            let result = self.api.create_pipeline_config(&pipeline_name, concourse_config, config_version).await;
            if result.is_err() {
                term::info!("Failed to create pipeline {} {:?}", pipeline_name, result);
            }

            term::info!("Unpausing pipeline {}", pipeline_name);
            let result = self.api.unpause_pipeline(&pipeline_name).await;
            if result.is_err() {
                return Err(anyhow::anyhow!("Failed to unpause pipeline {}", pipeline_name));
            }

            Ok(pipeline_name)
        })
    }

    fn run_pipeline(&mut self, pipeline_name: &PipelineName) -> Result<CIResult, anyhow::Error> {
        self.runtime.block_on(async {
            let concourse_url = &self.concourse_url.clone();
            let result = self.api.get_all_pipeline_jobs(pipeline_name)
                .await
                .map(|jobs| jobs.get(0).unwrap().get_name());
            if result.is_err() {
                return Err(anyhow!("Cannot find jobs for {} pipeline",pipeline_name));
            }

            let job_name = result.unwrap();

            let build_result = self.api.trigger_new_pipeline_job_build(pipeline_name, &job_name).await;
            if build_result.is_err() {
                return Err(anyhow!("Cannot trigger job {} build for {} pipeline", job_name, pipeline_name));
            }
            let build = build_result.unwrap();


            let watch_build_result = loop {
                sleep(Duration::from_secs(3)).await;
                let build_result = self.api.get_build(&build.id).await;
                match build_result {
                    Ok(build) => {
                        if build.has_completed() {
                            term::info!("Pipeline job build #{} has completed execution", build.id);
                            break Ok(build);
                        }
                    }
                    Err(error) => {
                        term::info!("Failed to get pipeline job build {:#?}", error);
                        break Err(anyhow::anyhow!("Failed to get pipeline job build"));
                    }
                }
            };

            watch_build_result.map(|build| {
                CIResult {
                    status: if build.has_completed_successfully() { CIResultStatus::Success } else { CIResultStatus::Failure },
                    url: format!("{}/teams/main/pipelines/{}/jobs/{}/builds/{}",
                                 concourse_url,
                                 build.pipeline_name,
                                 build.job_name,
                                 build.name,
                    ),
                }
            })
        })
    }
}
