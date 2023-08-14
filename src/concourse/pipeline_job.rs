use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct JobInputs {
    pub name: String,
    pub resource: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum JobStatus {
    Pending,
    Started,
    Succeeded,
    Unknown,
}

impl<'de> Deserialize<'de> for JobStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "pending" => JobStatus::Pending,
            "started" => JobStatus::Started,
            "succeeded" => JobStatus::Succeeded,
            _ => JobStatus::Unknown,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Build {
    pub id: usize,
    pub name: String,
    pub status: JobStatus,
    pub start_time: i64,
    pub end_time: i64,
    pub team_name: String,
    pub pipeline_id: usize,
    pub pipeline_name: String,
    pub job_name: String,
}

#[derive(Debug, Deserialize)]
pub struct TriggeredJob {
    pub id: usize,
    pub name: String,
    pub team_name: String,
    pub pipeline_id: usize,
    pub pipeline_name: String,
    pub has_new_inputs: Option<bool>,
    pub next_build: Build,
    pub inputs: Vec<JobInputs>,
}

#[derive(Debug, Deserialize)]
pub struct FinishedJob {
    pub id: usize,
    pub name: String,
    pub team_name: String,
    pub pipeline_id: usize,
    pub pipeline_name: String,
    pub finished_build: Build,
    pub transition_build: Build,
    pub inputs: Vec<JobInputs>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PipelineJob {
    TriggeredJob(TriggeredJob),
    FinishedJob(FinishedJob),
}

impl PipelineJob {
    pub fn has_finished_successful(&self) -> bool {
        match self {
            PipelineJob::FinishedJob(job) => {
                job.finished_build.status == JobStatus::Succeeded
            }
            _ => false
        }
    }

    pub fn get_status(&self) -> JobStatus {
        match self {
            PipelineJob::TriggeredJob(job) => job.next_build.status,
            PipelineJob::FinishedJob(job) => job.finished_build.status,
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            PipelineJob::TriggeredJob(job) => job.name.clone(),
            PipelineJob::FinishedJob(job) => job.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::concourse::pipeline_job::{Build, JobInputs, JobStatus, PipelineJob};

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
        assert_eq!(build.status, JobStatus::Pending);
        assert_eq!(build.start_time, -62135596800);
        assert_eq!(build.end_time, -62135596800);
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
        assert_eq!(build.status, JobStatus::Started);
        assert_eq!(build.start_time, 1690735633);
        assert_eq!(build.end_time, -62135596800);
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
        assert_eq!(build.status, JobStatus::Succeeded);
        assert_eq!(build.start_time, 1690735633);
        assert_eq!(build.end_time, 1690735639);
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
                assert_eq!(job.next_build.status, JobStatus::Pending);
            }
            PipelineJob::FinishedJob(_) => assert!(false, "expected triggered job"),
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
        }"#;

        let job = serde_json::from_str::<PipelineJob>(json)?;

        match job {
            PipelineJob::TriggeredJob(job) => {
                assert_eq!(job.id, 76);
                assert_eq!(job.name, "configure-pipeline");
                assert_eq!(job.team_name, "main");
                assert_eq!(job.pipeline_id, 70);
                assert_eq!(job.pipeline_name, "heartwood-configure");
                assert_eq!(job.next_build.status, JobStatus::Started);
            }
            PipelineJob::FinishedJob(_) => assert!(false, "expected triggered job"),
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
            PipelineJob::TriggeredJob(_) => {
                assert!(false, "expected finished job")
            }
            PipelineJob::FinishedJob(job) => {
                assert_eq!(job.id, 76);
                assert_eq!(job.name, "configure-pipeline");
                assert_eq!(job.team_name, "main");
                assert_eq!(job.pipeline_id, 70);
                assert_eq!(job.pipeline_name, "heartwood-configure");
                assert_eq!(job.finished_build.status, JobStatus::Succeeded);
                assert_eq!(job.transition_build.status, JobStatus::Succeeded);
            }
        }
        Ok(())
    }
}