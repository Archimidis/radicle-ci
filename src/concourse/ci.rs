use std::time::Duration;

use anyhow::Error;
use radicle_term as term;
use tokio::time::sleep;

use crate::ci::{CI, CIJob};
use crate::concourse::api::ConcourseAPI;
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

    pub async fn watch_configure_pipeline(&self) {
        loop {
            sleep(Duration::from_secs(3)).await;

            let result = self.api.get_all_jobs().await;
            match result {
                Ok(jobs) => {
                    println!("All jobs: {:#?}", jobs);
                    let job = jobs.get(0).unwrap();
                    println!("Configuration pipeline job {} status: {:?}", job.get_name(), job.get_status());


                    let job = jobs.iter().find(|pipeline_job| {
                        match pipeline_job {
                            PipelineJob::TriggeredJob(job) => job.name == "configure-pipeline",
                            PipelineJob::FinishedJob(job) => job.name == "configure-pipeline",
                            _ => false,
                        }
                    }).unwrap();

                    // job.has_finished_successful()
                    let has_finished_successful = jobs.iter().find(|pipeline_job| {
                        match pipeline_job {
                            PipelineJob::TriggeredJob(job) => job.name == "configure-pipeline",
                            PipelineJob::FinishedJob(job) => job.name == "configure-pipeline",
                            _ => false,
                        }
                    }).map(|job| job.has_finished_successful()).unwrap();

                    if has_finished_successful {
                        term::info!("Configuration pipeline job {} has finished successfully", job.get_name());
                        break;
                    }
                }
                Err(error) => term::info!("Failed to get all jobs {:#?}", error),
            }
        }
    }
}

impl CI for ConcourseCI {
    fn setup(&mut self, job: CIJob) -> Result<(), Error> {
        self.runtime.block_on(async {
            term::info!("Getting access token");
            let result = self.api.get_access_token().await;

            if let Ok(token) = result {
                term::info!("Access token acquired {}", token.access_token);
            } else {
                return Err(anyhow::anyhow!("Failed to get access token"));
            }

            term::info!("Creating the pipeline");
            let result = self.api.create_pipeline(&job).await;
            match result {
                Ok(()) => term::info!("Pipeline configuration creation triggered"),
                Err(error) => term::info!("Failed to trigger create pipeline configuration {:?}", error),
            }

            term::info!("Unpausing the pipeline");
            let result = self.api.unpause_pipeline(&job.project_id).await;
            if let Ok(job) = result {
                term::info!("Pipeline configuration unpaused {:?}", job);
            } else {
                return Err(anyhow::anyhow!("Failed to unpause pipeline configuration"));
            }

            Ok(())
        })
    }

    fn run_pipeline(&self, project_id: &String) -> Result<(), Error> {
        self.runtime.block_on(async {
            let result = self.api.trigger_pipeline_configuration(project_id).await;
            if let Ok(pipeline_configuration_job) = result {
                term::info!("Pipeline configuration triggered {:?}", pipeline_configuration_job);
            } else {
                return Err(anyhow::anyhow!("Failed to trigger pipeline configuration"));
            }

            self.watch_configure_pipeline().await;

            let result_job_name = self.api.get_all_pipeline_jobs(project_id).await
                .map(|jobs| jobs.get(0).unwrap().get_name());

            let build_result = match result_job_name {
                Ok(job_name) => self.api.trigger_new_pipeline_job_build(project_id, &job_name).await,
                Err(error) => {
                    term::info!("Failed to get trigger new pipeline job build {:#?}", error);
                    return Err(anyhow::anyhow!("Failed to get trigger new pipeline job build"));
                }
            };

            let build_id = match build_result {
                Ok(build) => build.id,
                Err(error) => {
                    term::info!("Failed to get pipeline job build {:#?}", error);
                    return Err(anyhow::anyhow!("Failed to get pipeline job build"));
                }
            };

            term::info!("[POLLING START] Build id: {}", build_id);
            let build = loop {
                sleep(Duration::from_secs(3)).await;
                let build_result = self.api.get_build(&build_id).await;
                match build_result {
                    Ok(build) => {
                        term::info!("Build {:#?}", build);
                        if build.has_succeeded() {
                            break build;
                        }
                    }
                    Err(error) => {
                        term::info!("Failed to get pipeline job build {:#?}", error);
                    }
                }
            };
            term::info!("[POLLING END]");

            term::info!("Build\n\tstatus: {:?}\n\tapi_url: {:?}", build.status, build.api_url);

            Ok(())
        })
    }
}
