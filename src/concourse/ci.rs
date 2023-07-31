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

            term::info!("start polling -------------------------------------");
            // TODO: Poll until pipeline configuration is completed
            loop {
                let result = self.api.get_all_jobs().await;
                match result {
                    Ok(jobs) => {
                        let job = jobs.get(0).unwrap();
                        match job {
                            PipelineJob::TriggeredJob(j) => println!("Triggered job {:?}", j.next_build.status),
                            PipelineJob::FinishedJob(j) => println!("Finished job {:?}", j.finished_build.status),
                        }

                        let maybe_job = jobs.iter().find(|pipeline_job| {
                            match pipeline_job {
                                PipelineJob::TriggeredJob(job) => job.name == "configure-pipeline",
                                PipelineJob::FinishedJob(job) => job.name == "configure-pipeline",
                            }
                        });

                        let has_finished_successful = maybe_job.map(|job| {
                            job.has_finished_successful()
                        }).unwrap();

                        if has_finished_successful {
                            println!("Build has finished successfully");
                            break;
                        }
                    }
                    Err(error) => println!("Failed to get all jobs {:#?}", error),
                }
                sleep(Duration::from_secs(3)).await;
            }
            term::info!("end polling -------------------------------------");

            let result = self.api.get_pipeline_jobs(project_id).await;
            println!("result {:#?}", result);
            match result {
                Ok(ref jobs) => {
                    let job_name = jobs.get(0).map(|job| job.name.clone()).unwrap();

                    let result = self.api.trigger_job(project_id, &job_name).await;
                    match result {
                        Ok(job) => term::info!("Job {} triggered", job.name),
                        Err(error) => term::info!("Unable to trigger job {:?}", error),
                    }
                }
                Err(error) => {
                    println!("Failed to get pipeline jobs {:?}", error);
                    return Err(anyhow::anyhow!("Failed to get pipeline jobs"));
                }
            }

            Ok(())
        })
    }
}
