mod kodi_helper;

use kodi_helper::Config;
use kodi_helper::Authorization;
use kodi_helper::rpc_call;

use serde_json::{json, Value};

use std::time::Duration;
use std::process::Command;
use std::io::{self, Write};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("config.yml")?;
    let base_url = &config.url;
    let auth = Authorization::new(&config.username, &config.password);

    let active_players_request_params = json!({
	    "jsonrpc": "2.0",
	    "method": "Player.GetActivePlayers",
	    "id": 1
    });

    let mut i = 0;

    loop {
        let active_players_response_json = rpc_call(&base_url, &auth, &active_players_request_params).await?;

        // Clone the JSON array to avoid borrowing issues
        let active_players = active_players_response_json["result"].as_array().unwrap_or(&vec![]).clone();

        if active_players.is_empty() {
            println!("\n[!] no show playing, calling other Rust binary...\n");

            // Call another Rust binary
            let status = Command::new("./your_other_rust_binary")
                .status()
                .expect("Failed to execute binary");

            if status.success() {
                println!("Successfully executed other Rust binary");
            } else {
                println!("Other Rust binary execution failed");
            }
        } else if i == 0 {
            print!("."); // Print a dot
	    io::stdout().flush()?; // Make sure the dot is immediately printed
            i = 60; // Reset the counter
        }

        i -= 1;

        sleep(Duration::from_secs(1)).await;
    }
}
