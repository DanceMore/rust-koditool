use serde_json::Value;
use serde::{Deserialize};
use reqwest::Client;
use reqwest::header::{HeaderValue, HeaderMap};
use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn load(filename: &str) -> Result<Self, Box<dyn Error>> {
        let config_content = fs::read_to_string(filename)?;
        let config: Config = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }
}

#[derive(Debug)]
pub struct Authorization {
    value: HeaderValue,
}

impl Authorization {
    pub fn new(username: &str, password: &str) -> Self {
        let auth_header_value = format!("Basic {}", base64::encode(format!("{}:{}", username, password)))
            .parse()
            .expect("failed to create Authorization header");

        Authorization {
            value: auth_header_value,
        }
    }
}

pub async fn rpc_call(base_url: &str, auth: &Authorization, request_params: &Value) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, auth.value.clone());

    let url = format!("{}/jsonrpc", base_url);

    let response = client
        .post(&url)
        .headers(headers)
        .body(reqwest::Body::from(request_params.to_string())) // Use reqwest::Body
        .send()
        .await?;


    // Read response body as bytes and deserialize using serde_json
    let response_bytes: Vec<u8> = response.bytes().await?.to_vec();
    let response_str = String::from_utf8_lossy(&response_bytes);
    let response_json: Result<Value, serde_json::Error> = serde_json::from_str(&response_str);

    match response_json {
        Ok(json) => Ok(json), // Return the JSON value
        Err(err) => Err(Box::new(err)), // Wrap the error in a Box
    }
}
