use std::io::Read;

use hyper::{Body, Client, Request, Response};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE};
use serde::Deserialize;

use crate::ci::{BuildName, JobName, PipelineConfig, PipelineName};
use crate::concourse::build::{Build, BuildID};
use crate::concourse::pipeline::Pipeline;
use crate::concourse::pipeline_job::PipelineJob;
use crate::concourse::response_error::ResponseError;
use crate::concourse::token::Token;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


async fn deserialize_json_response<T>(response: Response<Body>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
{
    let body = hyper::body::aggregate(response).await?;
    let result: T = serde_json::from_reader(body.reader())?;
    Ok(result)
}

async fn deserialize_string_response(response: Response<Body>) -> Result<String> {
    let content_length = response.headers().get(CONTENT_LENGTH).unwrap().to_str()?.parse::<usize>().unwrap();
    let body = hyper::body::aggregate(response).await?;
    let mut dst = vec![0; content_length];
    let num = body.reader().read(&mut dst)?;
    let result = std::str::from_utf8(&dst[..num])?;
    Ok(result.to_string())
}

#[derive(Clone)]
pub struct ConcourseAPI {
    client: Client<HttpConnector>,
    ci_pass: String,
    ci_user: String,
    concourse_uri: String,
    token: Option<Token>,
}

impl ConcourseAPI {
    pub fn new(concourse_uri: String, ci_user: String, ci_pass: String) -> ConcourseAPI {
        ConcourseAPI {
            client: Client::new(),
            concourse_uri,
            ci_user,
            ci_pass,
            token: None,
        }
    }

    /// Obtain credentials. This is how we get the access token required for all other API requests.
    /// Notice the Authorization header. Its value is always the same and was found in the Concourse
    /// repository.
    pub async fn get_access_token(&mut self) -> Result<Token> {
        let path = "/sky/issuer/token";

        let request = Request::builder()
            .method("POST")
            .uri(format!("{}{}", self.concourse_uri, path))
            .header(AUTHORIZATION, "Basic Zmx5OlpteDU=")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!("grant_type=password&username={}&password={}&scope=openid%20profile%20email%20federated:id%20groups", self.ci_user, self.ci_pass).into())?;

        let response = self.client.request(request).await?;
        let token = deserialize_json_response::<Token>(response).await?;

        self.token = Some(token.clone());

        Ok(token)
    }

    /// Returns a list of all pipelines.
    pub async fn get_all_pipelines(&self) -> Result<Vec<Pipeline>> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/api/v1/pipelines", self.concourse_uri))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            let result = deserialize_json_response::<Vec<Pipeline>>(response).await?;
            Ok(result)
        }
    }

    /// Returns a list of all pipeline jobs.
    pub async fn get_all_jobs(&self) -> Result<Vec<PipelineJob>> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/api/v1/jobs", self.concourse_uri))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;


        let response = self.client.request(request).await?;
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            let result = deserialize_json_response::<Vec<PipelineJob>>(response).await?;
            Ok(result)
        }
    }

    /// Returns a specific pipeline job build.
    pub async fn get_build(&self, build_id: &BuildID) -> Result<Build> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request: Request<Body> = Request::builder()
            .method("GET")
            .uri(format!("{}/api/v1/builds/{}", self.concourse_uri, build_id))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        // Both 4xx and 5xx responses return a string body that cannot be parsed as JSON.
        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            deserialize_json_response::<Build>(response).await
        }
    }

    pub async fn get_pipeline(&self, pipeline_name: &PipelineName) -> Result<Pipeline> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}", self.concourse_uri, pipeline_name))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        // Both 4xx and 5xx responses return a string body that cannot be parsed as JSON.
        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            deserialize_json_response::<Pipeline>(response).await
        }
    }

    /// Create a new pipeline in concourse based on the configuration provided.
    pub async fn create_pipeline(&self, pipeline_name: &PipelineName, config: PipelineConfig) -> Result<()> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request = Request::builder()
            .method("PUT")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/config", self.concourse_uri, pipeline_name))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .header(CONTENT_TYPE, "application/x-yaml")
            .header("X-Concourse-Config-Version", "1")
            .body(config.into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        // Both 4xx and 5xx responses return a string body that cannot be parsed as JSON.
        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            Ok(())
        }
    }

    /// After the pipeline is created it is in a paused state. This method will unpause it making it
    /// available for execution.
    pub async fn unpause_pipeline(&self, pipeline_name: &PipelineName) -> Result<()> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request = Request::builder()
            .method("PUT")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/unpause", self.concourse_uri, pipeline_name))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        // Both 4xx and 5xx responses return a string body that cannot be parsed as JSON.
        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            Ok(())
        }
    }


    /// Get all pipeline jobs.
    pub async fn get_all_pipeline_jobs(&self, pipeline_name: &PipelineName) -> Result<Vec<PipelineJob>> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs", self.concourse_uri, pipeline_name))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            let result = deserialize_json_response::<Vec<PipelineJob>>(response).await?;
            Ok(result)
        }
    }

    /// Trigger a job belonging to a specific pipeline.
    pub async fn trigger_pipeline_job(&self, pipeline_name: &PipelineName, job_name: &JobName) -> Result<Build> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request: Request<Body> = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs/{}/builds", self.concourse_uri, pipeline_name, job_name))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        // Both 4xx and 5xx responses return a string body that cannot be parsed as JSON.
        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            deserialize_json_response::<Build>(response).await
        }
    }

    /// Trigger a new build for a specific pipeline job
    pub async fn trigger_new_pipeline_job_build(&self, pipeline_name: &PipelineName, job_name: &JobName) -> Result<Build> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs/{}/builds", self.concourse_uri, pipeline_name, job_name))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            let result = deserialize_json_response::<Build>(response).await?;
            Ok(result)
        }
    }


    /// Returns data for a specific pipeline job build.
    pub async fn get_pipeline_job_build(&self, pipeline_name: &PipelineName, job_name: &JobName, build_name: &BuildName) -> Result<Build> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs/{}/builds/{}", self.concourse_uri, pipeline_name, job_name, build_name))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            let result = deserialize_json_response::<Build>(response).await?;
            Ok(result)
        }
    }

    /// Returns a list of all builds in concourse related to a specific pipeline job.
    pub async fn get_all_pipeline_job_builds(&self, pipeline_name: &PipelineName, job_name: &JobName) -> Result<Vec<Build>> {
        let access_token = match &self.token {
            Some(token) => token.get_access_token()?,
            None => return Err(Box::new(ResponseError { errors: vec!["No access token acquired yet.".into()], warnings: None }))
        };

        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs/{}/builds", self.concourse_uri, pipeline_name, job_name))
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            let result = deserialize_json_response::<Vec<Build>>(response).await?;
            Ok(result)
        }
    }
}
