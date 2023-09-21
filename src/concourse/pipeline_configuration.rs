use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PlanStep {
    pub get: Option<String>,
    pub version: Option<String>,
    pub file: Option<String>,
    pub set_pipeline: Option<String>,
}


#[derive(Deserialize, Debug)]
pub struct PipelineJob {
    pub name: String,
    pub plan: Vec<PlanStep>,
}

#[derive(Deserialize, Debug)]
pub struct Source {
    pub branch: String,
    pub uri: String,
}

#[derive(Deserialize, Debug)]
pub struct Resource {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub source: Source,
    pub icon: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub resources: Option<Vec<Resource>>,
    pub jobs: Vec<PipelineJob>,
}

#[derive(Deserialize, Debug)]
pub struct PipelineConfiguration {
    pub config: Config,
    pub version: Option<String>,
}
