mod project;
mod commit;

use std::fmt::{Display, Formatter};

use hyper::{Client, Uri};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use serde_json::Value;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Debug, PartialEq)]
struct RadicleApiUrl(pub String);

impl Display for RadicleApiUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct RadicleAPI {
    client: Client<HttpsConnector<HttpConnector>>,
    radicle_uri: RadicleApiUrl,
}

impl RadicleAPI {
    pub fn new(radicle_api_url: RadicleApiUrl) -> RadicleAPI {
        let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

        RadicleAPI { client, radicle_uri: radicle_api_url }
    }

    /// `GET /projects/:project`
    pub async fn get_project(&self, project_id: &String) -> Result<Value> {
        let uri = format!("{}/projects/{}", self.radicle_uri, project_id);
        let response = self.client.get(uri.parse::<Uri>()?).await?;

        let body = hyper::body::to_bytes(response.into_body()).await?;
        let data: Value = serde_json::from_slice(&body)?;

        Ok(data)
    }

    /// `GET /projects/:project/patches/:id`
    pub async fn get_project_patch(&self, project_id: &String, patch_id: &String) -> Result<Value> {
        let uri = format!("{}/projects/{}/patches/{}", self.radicle_uri, project_id, patch_id);
        let response = self.client.get(uri.parse::<Uri>()?).await?;

        let body = hyper::body::to_bytes(response.into_body()).await?;
        let data: Value = serde_json::from_slice(&body)?;

        Ok(data)
    }

    /// `GET /projects/:project/commits/:id`
    pub async fn get_project_commit(&self, project_id: &String, commit_id: &String) -> Result<Value> {
        let uri = format!("{}/projects/{}/commits/{}", self.radicle_uri, project_id, commit_id);
        let response = self.client.get(uri.parse::<Uri>()?).await?;

        let body = hyper::body::to_bytes(response.into_body()).await?;
        let data: Value = serde_json::from_slice(&body)?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // const PROJECT_ID: &str = "rad:z3gqcJUoA1n9HaHKufZs5FCSGazv5";
    const PROJECT_ID: &str = "rad:z5K7afvETkdZAtQ9VDgNsj8KroK8";

    // const URI: &str = "https://seed.radicle.xyz/api/v1";
    const URI: &str = "http://127.0.0.1:8778/api/v1";
    const PATCH_ID: &str = "a4c5345fe1ead88676a849b2522cc0b207d532a6";
    const COMMIT_ID: &str = "5bd72d8f3492dd218b101cc09f03b9dda1761514";

    #[tokio::test]
    async fn test_get_projects() {
        let api = RadicleAPI::new(RadicleApiUrl(String::from(URI)));
        let result = api.get_project(&String::from(PROJECT_ID)).await;

        println!("{:#?}", result);
    }

    #[tokio::test]
    async fn test_get_project_patch() {
        let api = RadicleAPI::new(RadicleApiUrl(String::from(URI)));
        let result = api.get_project_patch(&String::from(PROJECT_ID), &String::from(PATCH_ID)).await;

        println!("{:#?}", result);
    }

    #[tokio::test]
    async fn test_get_project_commit() {
        let api = RadicleAPI::new(RadicleApiUrl(String::from(URI)));
        let result = api.get_project_commit(&String::from(PROJECT_ID), &String::from(COMMIT_ID)).await;

        println!("{:#?}", result);
    }
}