use std::str;
use std::time::Duration;

use anyhow::anyhow;
use radicle_term as term;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::time::sleep;

use crate::ci::{CI, CIJob, CIResult, CIResultStatus, PipelineConfig, PipelineName};
use crate::concourse::api::ConcourseAPI;
use crate::concourse::build::{Build, BuildID};
use crate::concourse::pipeline_job::PipelineJob;

pub struct ConcourseCI {
    runtime: tokio::runtime::Runtime,
    api: ConcourseAPI,
}

impl Clone for ConcourseCI {
    fn clone(&self) -> Self {
        Self {
            // TODO: Investigate if this is the right way to clone a runtime
            runtime: tokio::runtime::Runtime::new().unwrap(),
            api: self.api.clone(),
        }
    }
}

impl ConcourseCI {
    // TODO: Create and use a CIConfig struct instead of passing individual parameters
    pub fn new(concourse_uri: String, ci_user: String, ci_pass: String) -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let api = ConcourseAPI::new(concourse_uri, ci_user, ci_pass);

        Self { runtime, api }
    }

    pub async fn watch_configure_pipeline(&self) -> Result<PipelineJob, anyhow::Error> {
        loop {
            sleep(Duration::from_secs(3)).await;

            let result = self.api.get_all_jobs()
                .await
                .map(|jobs| {
                    println!("All jobs: {:#?}", jobs);
                    // It is safe to unwrap the result since a pipeline job will exist with the name
                    // "configure-pipeline" for sure. This is how the concourse pipelines are
                    // declared.
                    let job = jobs.iter().find(|job| job.is_named(&String::from("pipeline-configure"))).unwrap();
                    println!("Configuration pipeline job {} status: {:?}", job.get_name(), job.get_status());
                    (*job).clone()
                });

            match result {
                Ok(pipeline_job) => {
                    if pipeline_job.has_completed() {
                        term::info!("Configuration pipeline job has completed execution");
                        break Ok(pipeline_job);
                    }
                }
                Err(error) => {
                    term::info!("Failed to get all jobs {:#?}", error);
                    break Err(anyhow::anyhow!("Failed to get all jobs"));
                }
            }
        }
    }

    pub async fn watch_pipeline_job_build(&self, build_id: BuildID) -> Result<Build, anyhow::Error> {
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

async fn load_concourse_config_template() -> Result<String, anyhow::Error> {
    let path = ".concourse/config.yaml";
    let mut file = File::open(path)
        .await
        .map_err(|_| anyhow::anyhow!("Unable to open file {path}"))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    str::from_utf8(&buffer)
        .map_err(|_| anyhow::anyhow!("Error parsing file"))
        .and_then(|x| Ok(String::from(x)))
}

fn create_concourse_pipeline_config(config: String, job: &CIJob) -> PipelineConfig {
    config
        .replace("((repo_url))", job.git_uri.as_str())
        .replace("((patch_revision_id))", job.patch_revision_id.as_str())
        .replace("((patch_head))", job.patch_head.as_str())
}

impl CI for ConcourseCI {
    fn setup(&mut self, job: CIJob) -> Result<PipelineName, anyhow::Error> {
        self.runtime.block_on(async {
            term::info!("Loading concourse configuration file");
            let concourse_config = load_concourse_config_template()
                .await
                .map(|template| create_concourse_pipeline_config(template, &job))?;
            let pipeline_name: PipelineName = format!("{}-pipeline", job.project_id);

            term::info!("Getting access token");
            let result = self.api.get_access_token().await;

            if let Ok(token) = result {
                term::info!("Access token acquired {}", token.access_token);
            } else {
                return Err(anyhow::anyhow!("Failed to get access token"));
            }

            term::info!("Creating the pipeline {}", pipeline_name);
            let result = self.api.create_pipeline(&pipeline_name, concourse_config).await;
            match result {
                Ok(()) => term::info!("Pipeline configuration creation triggered"),
                Err(error) => term::info!("Failed to trigger create pipeline configuration {:?}", error),
            }

            term::info!("Unpausing the pipeline");
            let result = self.api.unpause_pipeline(&pipeline_name).await;
            if let Ok(job) = result {
                term::info!("Pipeline configuration unpaused {:?}", job);
            } else {
                return Err(anyhow::anyhow!("Failed to unpause pipeline configuration"));
            }

            Ok(pipeline_name)
        })
    }

    fn run_pipeline(&self, pipeline_name: &PipelineName) -> Result<CIResult, anyhow::Error> {
        // TODO: watch for pipeline creation
        self.runtime.block_on(async {
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

            self.watch_pipeline_job_build(build.id)
                .await
                .map(|build| {
                    CIResult {
                        status: if build.has_completed_successfully() { CIResultStatus::Success } else { CIResultStatus::Failure },
                        url: format!("http://localhost:8080/teams/main/pipelines/{}/jobs/{}/builds/{}",
                                     build.pipeline_name,
                                     build.job_name,
                                     build.name,
                        ),
                    }
                })
        })
    }
}
