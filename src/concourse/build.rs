use serde::{Deserialize, Deserializer};

type BuildID = usize;
type PipelineID = usize;

#[derive(Debug, PartialEq)]
pub enum BuildStatus {
    Pending,
    Started,
    Succeeded,
    Unknown(String),
}

impl<'de> Deserialize<'de> for BuildStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "pending" => BuildStatus::Pending,
            "started" => BuildStatus::Started,
            "succeeded" => BuildStatus::Succeeded,
            unknown => BuildStatus::Unknown(unknown.into()),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Build {
    pub id: BuildID,
    pub team_name: String,
    pub name: String,
    pub status: BuildStatus,
    pub api_url: String,
    pub job_name: String,
    pub pipeline_id: PipelineID,
    pub pipeline_name: String,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub created_by: String,
}

#[cfg(test)]
mod tests {
    use crate::concourse::build::{Build, BuildStatus};

    #[test]
    fn will_successfully_deserialize_a_build() -> Result<(), serde_json::Error> {
        let json = r#"
        {
            "id": 3094,
            "team_name": "main",
            "name": "4",
            "status": "succeeded",
            "api_url": "/api/v1/builds/3094",
            "job_name": "poc-job",
            "pipeline_id": 101,
            "pipeline_name": "heartwood",
            "start_time": 1692021331,
            "end_time": 1692021336,
            "created_by": "test"
        }"#;

        let build = serde_json::from_str::<Build>(json)?;

        assert_eq!(build.id, 3094);
        assert_eq!(build.team_name, "main");
        assert_eq!(build.name, "4");
        assert_eq!(build.status, BuildStatus::Succeeded);
        assert_eq!(build.api_url, "/api/v1/builds/3094");
        assert_eq!(build.job_name, "poc-job");
        assert_eq!(build.pipeline_id, 101);
        assert_eq!(build.pipeline_name, "heartwood");
        assert_eq!(build.start_time, Some(1692021331));
        assert_eq!(build.end_time, Some(1692021336));
        assert_eq!(build.created_by, "test");

        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_succeeded_build() -> Result<(), serde_json::Error> {
        let json = r#"
        {
            "id": 3094,
            "team_name": "main",
            "name": "4",
            "status": "succeeded",
            "api_url": "/api/v1/builds/3094",
            "job_name": "poc-job",
            "pipeline_id": 101,
            "pipeline_name": "heartwood",
            "start_time": 1692021331,
            "end_time": 1692021336,
            "created_by": "test"
        }"#;

        let build = serde_json::from_str::<Build>(json)?;

        assert_eq!(build.status, BuildStatus::Succeeded);

        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_started_build() -> Result<(), serde_json::Error> {
        let json = r#"
        {
            "id": 3094,
            "team_name": "main",
            "name": "4",
            "status": "started",
            "api_url": "/api/v1/builds/3094",
            "job_name": "poc-job",
            "pipeline_id": 101,
            "pipeline_name": "heartwood",
            "start_time": 1692021331,
            "end_time": 1692021336,
            "created_by": "test"
        }"#;

        let build = serde_json::from_str::<Build>(json)?;

        assert_eq!(build.status, BuildStatus::Started);

        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_pending_build() -> Result<(), serde_json::Error> {
        let json = r#"
        {
            "id": 3094,
            "team_name": "main",
            "name": "4",
            "status": "pending",
            "api_url": "/api/v1/builds/3094",
            "job_name": "poc-job",
            "pipeline_id": 101,
            "pipeline_name": "heartwood",
            "start_time": 1692021331,
            "end_time": 1692021336,
            "created_by": "test"
        }"#;

        let build = serde_json::from_str::<Build>(json)?;

        assert_eq!(build.status, BuildStatus::Pending);

        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_an_unknown_status_build() -> Result<(), serde_json::Error> {
        let json = r#"
        {
            "id": 3094,
            "team_name": "main",
            "name": "4",
            "status": "unknown-status",
            "api_url": "/api/v1/builds/3094",
            "job_name": "poc-job",
            "pipeline_id": 101,
            "pipeline_name": "heartwood",
            "start_time": 1692021331,
            "end_time": 1692021336,
            "created_by": "test"
        }"#;

        let build = serde_json::from_str::<Build>(json)?;

        assert_eq!(build.status, BuildStatus::Unknown(String::from("unknown-status")));

        Ok(())
    }
}
