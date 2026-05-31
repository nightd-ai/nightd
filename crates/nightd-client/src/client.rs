use reqwest::ClientBuilder;
use serde::Deserialize;
use std::env;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct InfoResponse {
    pub status: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        let socket_path = env::var("NIGHTD_SOCKET_PATH")
            .expect("NIGHTD_SOCKET_PATH environment variable must be set");

        let client = ClientBuilder::new().unix_socket(socket_path).build()?;

        Ok(Client { client })
    }

    pub async fn info(&self) -> Result<InfoResponse, Error> {
        let response = self.client.get("http://localhost/info").send().await?;

        let info = response.json::<InfoResponse>().await?;
        Ok(info)
    }
}
