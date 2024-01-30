use scraper::{Html, Selector};
use quoted_printable::{decode, ParseMode};
use std::collections::HashSet;
use serde::{Deserialize};
use std::fs::File;
use std::io::Read;
use serde_json::json;

#[derive(Deserialize)]
struct Config {
    email: String,
    password: String,
}

pub async fn send_webhook_with_embed(url: &str, link: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let color = 16711680; 
    let embed = json!({
        "title": "New PayPal Links",
        "description": link,
        "color": color,
        "footer": {
            "text": "PayPal Links",
            "icon_url": "https://image.noelshack.com/fichiers/2023/35/2/1693332455-image-3.png"
        }
    });

    let payload = json!({
        "username": "FluxyIO Raffle",
        "avatar_url": "https://image.noelshack.com/fichiers/2023/35/2/1693332455-image-3.png",
        "embeds": [embed]
    });

    let res = client.post(url).json(&payload).send().await?;
    res.error_for_status()?;

    Ok(())
}

pub fn extract_links_from_email_body(body: &str) -> Vec<String> {
    let decoded_bytes = decode(body.as_bytes(), ParseMode::Robust).unwrap();
    let decoded_body = String::from_utf8_lossy(&decoded_bytes).to_string();

    let document = Html::parse_document(&decoded_body);
    let link_selector = Selector::parse("a").unwrap();

    let mut links = Vec::new();
    for link_element in document.select(&link_selector) {
        if let Some(link) = link_element.value().attr("href") {
            if link.contains("paypal") {
                links.push(link.to_string());
            }
        }
    }

    links
}

pub fn fetch_emails_from_sender(sender: &str, seen: &mut HashSet<String>) -> imap::error::Result<Vec<String>> {
    let domain = "imap.gmail.com";
    let tls = native_tls::TlsConnector::builder().build().unwrap();
    let client = imap::connect((domain, 993), domain, &tls).unwrap();

    let mut config_file = File::open("config.json").expect("Failed to open config.json");
    let mut config_content = String::new();
    config_file.read_to_string(&mut config_content).expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config_content).expect("Failed to parse config.json");

    let mut imap_session = client.login(&config.email, &config.password).unwrap();

    imap_session.select("INBOX").unwrap();

    let now = chrono::Utc::now();
    let thirty_minutes_ago = (now - chrono::Duration::minutes(30)).format("%d-%b-%Y").to_string();
    let query = format!("FROM {} SINCE {}", sender, thirty_minutes_ago);
    let search_results = imap_session.search(&query)?;

    let mut email_bodies = Vec::new();

    for seq in search_results.iter() {
        if seen.contains(seq.to_string().as_str()) {
            continue;
        }

        let messages = imap_session.fetch(seq.to_string(), "RFC822")?;
        let message = if let Some(m) = messages.iter().next() {
            m
        } else {
            continue;
        };

        let body = std::str::from_utf8(message.body().unwrap())
            .expect("message was not valid utf-8")
            .to_string();

        email_bodies.push(body);
        seen.insert(seq.to_string());
    }

    imap_session.logout().unwrap();

    Ok(email_bodies)
}