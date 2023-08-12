use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use reqwest::Client;
use reqwest::header::{HeaderValue, HeaderMap};
use std::error::Error;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration from YAML
    let config = load_config("config.yml")?;
    let base_url = &config.url;
    let auth = Authorization::new(&config.username, &config.password);

    let request_params = json!({
        "jsonrpc": "2.0",
        "method": "VideoLibrary.GetEpisodes",
        "params": {
            "properties": ["title", "season", "episode"],
            "limits": { "start": 0, "end": 10 }
        },
        "id": 1
    });

    let response_json = rpc_call(&base_url, &auth, &request_params).await?;

    println!("[+] Response: {:?}", response_json);

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Config {
    url: String,
    username: String,
    password: String,
}

#[derive(Debug)]
struct Authorization {
    value: HeaderValue,
}

impl Authorization {
    fn new(username: &str, password: &str) -> Self {
        let auth_header_value = format!("Basic {}", base64::encode(format!("{}:{}", username, password)))
            .parse()
            .expect("failed to create Authorization header");
 
        Authorization {
            value: auth_header_value,
        }
    }
}

fn load_config(filename: &str) -> Result<Config, Box<dyn Error>> {
    let config_content = fs::read_to_string(filename)?;
    let config: Config = serde_yaml::from_str(&config_content)?;
    Ok(config)
}

async fn rpc_call(base_url: &str, auth: &Authorization, request_params: &Value) -> Result<Value, Box<dyn Error>> {
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
