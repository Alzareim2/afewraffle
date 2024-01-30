use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::error::Error;
use mac_address::get_mac_address;

#[derive(Serialize, Deserialize)]
struct KeyInput {
    key: String,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    status: String,
    message: String,
}

pub async fn check_login(api_key: &str) -> Result<(bool, String), Box<dyn Error>> {
    let url = "https://fluxy.api-scale.com:8085/external_data2";

    let hwid = get_mac_address()?.ok_or("No MAC address found")?.to_string();
    
    let key_input = KeyInput { key: api_key.to_string() };

    let client = reqwest::Client::new();
    let response = client.post(url)
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/json")
        .json(&key_input)
        .send().await?;

    if response.status().is_success() {
        let api_response: ApiResponse = response.json().await?;
        if api_response.status == "success" {
            Ok((true, hwid)) // or any other success information you need
        } else {
            Ok((false, api_response.message))
        }
    } else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Non-success status code received")))
    }
}


use colored::Colorize;
use crate::model::Config;


pub async fn load_api_key() -> Result<String, Box<dyn std::error::Error>> {
    match tokio::fs::read_to_string("config.json").await {
        Ok(contents) => {
            let config: Config = serde_json::from_str(&contents)?;
            Ok(config.api_key)
        },
        Err(_) => {
            println!("{}", "Please enter your API key:".bold().bright_white());
            let mut api_key = String::new();
            std::io::stdin().read_line(&mut api_key)?;
            api_key = api_key.trim().to_string();

            let config = Config { api_key: api_key.clone() };
            let config_contents = serde_json::to_string(&config)?;
            tokio::fs::write("config.json", config_contents).await?;

            Ok(api_key)
        }
    }
}


#[derive(Serialize, Deserialize)]
struct ErrorLog {
    username: String,
    error_message: String,
}
