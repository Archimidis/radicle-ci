use serde::Deserialize;

type PipelineID = usize;
type BuildID = usize;
type JobID = usize;

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    pub id: PipelineID,
    pub name: String,
    pub paused: bool,
    pub public: bool,
    pub archived: bool,
    pub team_name: String,
    pub last_updated: i64,
    pub parent_build_id: Option<BuildID>,
    pub parent_job_id: Option<JobID>,
}

#[cfg(test)]
mod tests {
    use crate::concourse::pipeline::Pipeline;

    #[test]
    fn will_successfully_deserialize_pipeline() -> Result<(), serde_json::Error> {
        let json = r#"
        {
            "id": 101,
            "name": "heartwood",
            "paused": false,
            "public": false,
            "archived": false,
            "team_name": "main",
            "last_updated": 1692021169
        }"#;

        let pipeline = serde_json::from_str::<Pipeline>(json)?;

        assert_eq!(pipeline.id, 101);
        assert_eq!(pipeline.name, "heartwood");
        assert_eq!(pipeline.paused, false);
        assert_eq!(pipeline.public, false);
        assert_eq!(pipeline.archived, false);
        assert_eq!(pipeline.team_name, "main");
        assert_eq!(pipeline.last_updated, 1692021169);
        assert_eq!(pipeline.parent_build_id, None);
        assert_eq!(pipeline.parent_job_id, None);

        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_pipeline_with_parent() -> Result<(), serde_json::Error> {
        let json = r#"
        {
            "id": 101,
            "name": "heartwood",
            "paused": false,
            "public": false,
            "archived": false,
            "team_name": "main",
            "parent_build_id": 3156,
            "parent_job_id": 106,
            "last_updated": 1692021169
        }"#;

        let pipeline = serde_json::from_str::<Pipeline>(json)?;

        assert_eq!(pipeline.id, 101);
        assert_eq!(pipeline.name, "heartwood");
        assert_eq!(pipeline.paused, false);
        assert_eq!(pipeline.public, false);
        assert_eq!(pipeline.archived, false);
        assert_eq!(pipeline.team_name, "main");
        assert_eq!(pipeline.last_updated, 1692021169);
        assert_eq!(pipeline.parent_build_id, Some(3156));
        assert_eq!(pipeline.parent_job_id, Some(106));

        Ok(())
    }
}