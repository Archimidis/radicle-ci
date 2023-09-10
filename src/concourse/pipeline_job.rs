use serde::Deserialize;

use crate::ci::JobName;
use crate::concourse::build::{Build, BuildStatus};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct JobInputs {
    pub name: String,
    pub resource: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TriggeredJob {
    pub id: usize,
    pub name: JobName,
    pub team_name: String,
    pub pipeline_id: usize,
    pub pipeline_name: String,
    pub has_new_inputs: Option<bool>,
    pub next_build: Build,
    pub inputs: Option<Vec<JobInputs>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FinishedJob {
    pub id: usize,
    pub name: JobName,
    pub team_name: String,
    pub pipeline_id: usize,
    pub pipeline_name: String,
    pub finished_build: Build,
    pub transition_build: Build,
    pub inputs: Option<Vec<JobInputs>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Job {
    pub id: usize,
    pub name: JobName,
    pub team_name: String,
    pub pipeline_id: usize,
    pub pipeline_name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum PipelineJob {
    TriggeredJob(TriggeredJob),
    FinishedJob(FinishedJob),
    Job(Job),
}

impl PipelineJob {
    pub fn is_named(&self, job_name: &String) -> bool {
        match self {
            PipelineJob::TriggeredJob(job) => job.name == *job_name,
            PipelineJob::FinishedJob(job) => job.name == *job_name,
            PipelineJob::Job(job) => job.name == *job_name,
        }
    }

    pub fn has_completed_successfully(&self) -> bool {
        match self {
            PipelineJob::FinishedJob(job) => job.finished_build.has_completed_successfully(),
            _ => false
        }
    }

    pub fn has_completed(&self) -> bool {
        match self {
            PipelineJob::TriggeredJob(_) => false,
            PipelineJob::FinishedJob(job) => job.finished_build.has_completed(),
            PipelineJob::Job(_) => true,
        }
    }

    pub fn get_status(&self) -> BuildStatus {
        match self {
            PipelineJob::TriggeredJob(job) => job.next_build.status.clone(),
            PipelineJob::FinishedJob(job) => job.finished_build.status.clone(),
            _ => BuildStatus::Unknown(String::from("unknown")),
        }
    }

    pub fn get_name(&self) -> JobName {
        match self {
            PipelineJob::TriggeredJob(job) => job.name.clone(),
            PipelineJob::FinishedJob(job) => job.name.clone(),
            PipelineJob::Job(job) => job.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::concourse::pipeline_job::{Build, BuildStatus, JobInputs, PipelineJob};

    #[test]
    fn will_successfully_deserialize_job_inputs() -> Result<(), serde_json::Error> {
        let json = r#"
        {
          "name": "heartwood",
          "resource": "heartwood-resource"
        }"#;

        let job_inputs = serde_json::from_str::<JobInputs>(json)?;

        assert_eq!(job_inputs.name, "heartwood");
        assert_eq!(job_inputs.resource, "heartwood-resource");
        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_pending_build() -> Result<(), serde_json::Error> {
        let json = r#"
        {
          "id": 2844,
          "name": "1",
          "status": "pending",
          "start_time": -62135596800,
          "end_time": -62135596800,
          "team_name": "main",
          "pipeline_id": 70,
          "pipeline_name": "heartwood-configure",
          "job_name": "configure-pipeline"
        }
        "#;

        let build = serde_json::from_str::<Build>(json)?;

        assert_eq!(build.id, 2844);
        assert_eq!(build.name, "1");
        assert_eq!(build.status, BuildStatus::Pending);
        assert_eq!(build.start_time, Some(-62135596800));
        assert_eq!(build.end_time, Some(-62135596800));
        assert_eq!(build.team_name, "main");
        assert_eq!(build.pipeline_id, 70);
        assert_eq!(build.pipeline_name, "heartwood-configure");
        assert_eq!(build.job_name, "configure-pipeline");
        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_started_build() -> Result<(), serde_json::Error> {
        let json = r#"
        {
          "id": 2844,
          "name": "1",
          "status": "started",
          "start_time": 1690735633,
          "end_time": -62135596800,
          "team_name": "main",
          "pipeline_id": 70,
          "pipeline_name": "heartwood-configure",
          "job_name": "configure-pipeline"
        }
        "#;

        let build = serde_json::from_str::<Build>(json)?;

        assert_eq!(build.id, 2844);
        assert_eq!(build.name, "1");
        assert_eq!(build.status, BuildStatus::Started);
        assert_eq!(build.start_time, Some(1690735633));
        assert_eq!(build.end_time, Some(-62135596800));
        assert_eq!(build.team_name, "main");
        assert_eq!(build.pipeline_id, 70);
        assert_eq!(build.pipeline_name, "heartwood-configure");
        assert_eq!(build.job_name, "configure-pipeline");
        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_succeeded_build() -> Result<(), serde_json::Error> {
        let json = r#"
        {
          "id": 2844,
          "name": "1",
          "status": "succeeded",
          "start_time": 1690735633,
          "end_time": 1690735639,
          "team_name": "main",
          "pipeline_id": 70,
          "pipeline_name": "heartwood-configure",
          "job_name": "configure-pipeline"
        }
        "#;

        let build = serde_json::from_str::<Build>(json)?;

        assert_eq!(build.id, 2844);
        assert_eq!(build.name, "1");
        assert_eq!(build.status, BuildStatus::Succeeded);
        assert_eq!(build.start_time, Some(1690735633));
        assert_eq!(build.end_time, Some(1690735639));
        assert_eq!(build.team_name, "main");
        assert_eq!(build.pipeline_id, 70);
        assert_eq!(build.pipeline_name, "heartwood-configure");
        assert_eq!(build.job_name, "configure-pipeline");
        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_pending_triggered_job() -> Result<(), serde_json::Error> {
        let json = r#"
        {
          "id": 76,
          "name": "configure-pipeline",
          "team_name": "main",
          "pipeline_id": 70,
          "pipeline_name": "heartwood-configure",
          "next_build": {
            "id": 2844,
            "name": "1",
            "status": "pending",
            "start_time": -62135596800,
            "end_time": -62135596800,
            "team_name": "main",
            "pipeline_id": 70,
            "pipeline_name": "heartwood-configure",
            "job_name": "configure-pipeline"
          },
          "inputs": [{
              "name": "heartwood",
              "resource": "heartwood"
          }]
        }
        "#;

        let job = serde_json::from_str::<PipelineJob>(json)?;

        match job {
            PipelineJob::TriggeredJob(job) => {
                assert_eq!(job.id, 76);
                assert_eq!(job.name, "configure-pipeline");
                assert_eq!(job.team_name, "main");
                assert_eq!(job.pipeline_id, 70);
                assert_eq!(job.pipeline_name, "heartwood-configure");
                assert_eq!(job.next_build.status, BuildStatus::Pending);
            }
            _ => assert!(false, "expected triggered job"),
        }
        Ok(())
    }

    #[test]
    fn deserialize_started_triggered_job() -> Result<(), serde_json::Error> {
        let json = r#"
        {
          "id": 76,
          "name": "configure-pipeline",
          "team_name": "main",
          "pipeline_id": 70,
          "pipeline_name": "heartwood-configure",
          "next_build": {
            "id": 2844,
            "name": "1",
            "status": "started",
            "start_time": -62135596800,
            "end_time": -62135596800,
            "team_name": "main",
            "pipeline_id": 70,
            "pipeline_name": "heartwood-configure",
            "job_name": "configure-pipeline"
          },
          "inputs": [{
            "name": "heartwood",
            "resource": "heartwood"
          }]
        }"#;

        let job = serde_json::from_str::<PipelineJob>(json)?;

        match job {
            PipelineJob::TriggeredJob(job) => {
                assert_eq!(job.id, 76);
                assert_eq!(job.name, "configure-pipeline");
                assert_eq!(job.team_name, "main");
                assert_eq!(job.pipeline_id, 70);
                assert_eq!(job.pipeline_name, "heartwood-configure");
                assert_eq!(job.next_build.status, BuildStatus::Started);
            }
            _ => assert!(false, "expected triggered job"),
        }
        Ok(())
    }

    #[test]
    fn deserialize_finished_job() -> Result<(), serde_json::Error> {
        let json = r#"
        {
          "id": 76,
          "name": "configure-pipeline",
          "team_name": "main",
          "pipeline_id": 70,
          "pipeline_name": "heartwood-configure",
          "has_new_inputs": true,
          "finished_build": {
            "id": 2844,
            "name": "1",
            "status": "succeeded",
            "start_time": 1690735633,
            "end_time": 1690735639,
            "team_name": "main",
            "pipeline_id": 70,
            "pipeline_name": "heartwood-configure",
            "job_name": "configure-pipeline"
          },
          "transition_build": {
            "id": 2844,
            "name": "1",
            "status": "succeeded",
            "start_time": 1690735633,
            "end_time": 1690735639,
            "team_name": "main",
            "pipeline_id": 70,
            "pipeline_name": "heartwood-configure",
            "job_name": "configure-pipeline"
          },
          "inputs": [{
            "name": "heartwood",
            "resource": "heartwood"
          }]
        }"#;

        let job = serde_json::from_str::<PipelineJob>(json)?;

        match job {
            PipelineJob::FinishedJob(job) => {
                assert_eq!(job.id, 76);
                assert_eq!(job.name, "configure-pipeline");
                assert_eq!(job.team_name, "main");
                assert_eq!(job.pipeline_id, 70);
                assert_eq!(job.pipeline_name, "heartwood-configure");
                assert_eq!(job.finished_build.status, BuildStatus::Succeeded);
                assert_eq!(job.transition_build.status, BuildStatus::Succeeded);
            }
            _ => assert!(false, "expected finished job"),
        }

        Ok(())
    }

    #[test]
    fn deserialize_finished_job_with_missing_inputs() -> Result<(), serde_json::Error> {
        let json = r#"
        {
          "id": 76,
          "name": "configure-pipeline",
          "team_name": "main",
          "pipeline_id": 70,
          "pipeline_name": "heartwood-configure",
          "has_new_inputs": true,
          "finished_build": {
            "id": 2844,
            "name": "1",
            "status": "succeeded",
            "start_time": 1690735633,
            "end_time": 1690735639,
            "team_name": "main",
            "pipeline_id": 70,
            "pipeline_name": "heartwood-configure",
            "job_name": "configure-pipeline"
          },
          "transition_build": {
            "id": 2844,
            "name": "1",
            "status": "succeeded",
            "start_time": 1690735633,
            "end_time": 1690735639,
            "team_name": "main",
            "pipeline_id": 70,
            "pipeline_name": "heartwood-configure",
            "job_name": "configure-pipeline"
          }
        }"#;

        let job = serde_json::from_str::<PipelineJob>(json)?;

        match job {
            PipelineJob::FinishedJob(job) => {
                assert_eq!(job.id, 76);
                assert_eq!(job.name, "configure-pipeline");
                assert_eq!(job.team_name, "main");
                assert_eq!(job.pipeline_id, 70);
                assert_eq!(job.pipeline_name, "heartwood-configure");
                assert_eq!(job.finished_build.status, BuildStatus::Succeeded);
                assert_eq!(job.transition_build.status, BuildStatus::Succeeded);
                assert_eq!(job.inputs, None);
            }
            _ => assert!(false, "expected finished job"),
        }

        Ok(())
    }

    #[test]
    fn deserialize_generic_job() -> Result<(), serde_json::Error> {
        let json = r#"
        {
          "id": 76,
          "name": "configure-pipeline",
          "team_name": "main",
          "pipeline_id": 70,
          "pipeline_name": "heartwood-configure"
        }"#;

        let job = serde_json::from_str::<PipelineJob>(json)?;

        match job {
            PipelineJob::Job(job) => {
                assert_eq!(job.id, 76);
                assert_eq!(job.name, "configure-pipeline");
                assert_eq!(job.team_name, "main");
                assert_eq!(job.pipeline_id, 70);
                assert_eq!(job.pipeline_name, "heartwood-configure");
            }
            _ => assert!(false, "expected generic job"),
        }

        Ok(())
    }
}