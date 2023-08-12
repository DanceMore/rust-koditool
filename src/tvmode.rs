mod kodi_helper;

use kodi_helper::Config;
use kodi_helper::Authorization;
use kodi_helper::rpc_call;

use serde_json::{json, Value};

use std::time::Duration;
use std::process::Command;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from YAML
    let config = Config::load("config.yml")?;
    let base_url = &config.url;
    let auth = Authorization::new(&config.username, &config.password);

    loop {
        // Get active players
        let active_players_request_params = json!({
            "jsonrpc": "2.0",
            "method": "Player.GetActivePlayers",
            "id": 1
        });

        let active_players_response_json = rpc_call(&base_url, &auth, &active_players_request_params).await?;
        //let active_players = active_players_response_json["result"].as_array().unwrap_or(&vec![]);

        // Store the active_players array as a named variable
        //let active_players_array = active_players.clone();

        //if active_players.is_empty() {
        //    println!("\n[!] no show playing, calling other Rust binary...\n");

        //    // Call another Rust binary
        //    let status = Command::new("./your_other_rust_binary")
        //        .status()
        //        .expect("Failed to execute binary");

        //    if status.success() {
        //        println!("Successfully executed other Rust binary");
        //    } else {
        //        println!("Other Rust binary execution failed");
        //    }
        //}

        sleep(Duration::from_secs(1)).await;
    }
}
