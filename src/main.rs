use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use reqwest::Client;
use reqwest::header::{HeaderValue, HeaderMap};
use std::error::Error;
use std::fs;
use std::env;
use rand::prelude::SliceRandom;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration from YAML
    let config = load_config("config.yml")?;
    let base_url = &config.url;
    let auth = Authorization::new(&config.username, &config.password);

    // Get the TV show name from the command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
	    eprintln!("Usage: {} <TV Show Name>", args[0]);
	    return Ok(());
    }
    let tv_show_name = &args[1];

    println!("[-] target => {:?}", tv_show_name);

    // Fetch the list of TV shows
    let tv_shows_request_params = json!({
	    "jsonrpc": "2.0",
	    "method": "VideoLibrary.GetTVShows",
	    "params": {
		    "properties": ["title"],
		    "limits": { "start": 0, "end": 1000 }
	    },
	    "id": 1
    });

    let tv_shows_response_json = rpc_call(&base_url, &auth, &tv_shows_request_params).await?;

    // Extract the "tvshows" array from the "result" field
    let tv_shows = tv_shows_response_json["result"]["tvshows"]
	    .as_array()
	    .ok_or("TV shows not found in response")?;

    // Find the TV show with the given name
    let tv_show = tv_shows
	    .iter()
	    .find(|show| show["title"].as_str() == Some(tv_show_name))
	    .ok_or_else(|| format!("TV show {} not found", tv_show_name))?;

    println!("Selected TV Show: {:?}", tv_show);

    let tv_show_id = tv_show["tvshowid"].as_u64().ok_or("TV show ID not found")?;
    println!("Selected TV Show ID: {}", tv_show_id);

    // Fetch the list of episodes
    let episodes_request_params = json!({
	    "jsonrpc": "2.0",
	    "method": "VideoLibrary.GetEpisodes",
	    "params": {
		    "tvshowid": tv_show_id, // Use the TV show ID you obtained earlier
		    "properties": ["title", "season", "episode"],
		    "limits": { "start": 0, "end": 1000 }
	    },
	    "id": 1
    });

    let episodes_response_json = rpc_call(&base_url, &auth, &episodes_request_params).await?;

    println!("Episodes Response: {:?}", episodes_response_json);

    // Extract the "episodes" array from the "result" field
    let episodes = episodes_response_json["result"]["episodes"]
	    .as_array()
	    .ok_or("Episodes not found in response")?;

    for episode in episodes {
	    let episode_id = episode["episodeid"].as_u64().ok_or("Episode ID not found")?;
	    let episode_title = episode["title"].as_str().ok_or("Episode title not found")?;
	    let season_number = episode["season"].as_u64().ok_or("Season number not found")?;
	    let episode_number = episode["episode"].as_u64().ok_or("Episode number not found")?;

	    println!(
		    "Episode ID: {}, Title: {}, Season: {}, Episode: {}",
		    episode_id, episode_title, season_number, episode_number
		    );
    }

    // Extract the episode IDs from the episodes array
    let episode_ids: Vec<u64> = episodes
	    .iter()
	    .map(|episode| episode["episodeid"].as_u64().unwrap())
	    .collect();

    // Randomly select an episode ID
    let mut rng = rand::thread_rng();
    let random_episode_id = episode_ids.choose(&mut rng).ok_or("No episodes available")?;

    println!("Randomly selected episode ID: {:?}", random_episode_id);

    // Prepare the request parameters
    let episode_details_request_params = json!({
	    "jsonrpc": "2.0",
	    "method": "VideoLibrary.GetEpisodeDetails",
	    "params": {
		    "episodeid": random_episode_id,
		    "properties": ["file"] // You can also include other properties you need
	    },
	    "id": 1
    });

    // Make the RPC call
    let episode_details_response_json = rpc_call(&base_url, &auth, &episode_details_request_params).await?;

    // Extract the file path from the response
    let episode_file_path = episode_details_response_json["result"]["episodedetails"]["file"]
	    .as_str()
	    .ok_or("Episode file path not found in response")?;

    println!("[!] file path => {:?}", episode_file_path);


    // Prepare the request parameters for playing the episode
    let play_episode_request_params = json!({
	    "jsonrpc": "2.0",
	    "method": "Player.Open",
	    "params": {
		    "item": {
			    "file": episode_file_path
		    }
	    },
	    "id": 1
    });

    // Make the RPC call to play the episode
    let play_response = rpc_call(&base_url, &auth, &play_episode_request_params).await?;
    println!("Play response: {:?}", play_response);

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
