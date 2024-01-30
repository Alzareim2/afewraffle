use serde_json;

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub product: String,
    pub user: String,
    pub plan: String,
    pub email: String,
    pub status: String,
    pub valid: bool,
    pub cancel_at_period_end: bool,
    pub payment_processor: String,
    pub license_key: String,
    pub metadata: serde_json::Value,
    pub quantity: i32,
    pub wallet_address: Option<String>,
    pub custom_fields_responses: serde_json::Value,
    pub discord: DiscordUser,
    pub nft_tokens: Vec<serde_json::Value>,
    pub expires_at: Option<String>,
    pub renewal_period_start: Option<String>,
    pub renewal_period_end: Option<String>,
    pub created_at: i64,
    pub manage_url: String,
    pub affiliate_page_url: String,
    pub checkout_session: Option<String>,
    pub access_pass: String,
    pub deliveries: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
pub struct DiscordUser {
    
    pub id: String,
    pub username: String,
    pub image_url: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserActivity {
    pub Username: String,
    pub Status: String,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub api_key: String,
}

pub fn display_menu() {
    let header1 = format!("{:<25}", "RaffleSite");
    let header3 = format!("{:<25}", "Tool");

    let column1_option1 = format!("{:<25}", "[1] AfewRaffle");
    let column1_option2 = format!("{:<25}", "[2] SOON");
    let column1_option3 = format!("{:<25}", "[3] SOON");
    let column1_option4 = format!("{:<25}", "[4] SOON");

    let column3_option1 = format!("{:<25}", "[8] EXIT");
    let column3_option2 = format!("{:<25}", "[5] J1G");
    let column3_option3 = format!("{:<25}", "[6] PaypalGrabber");
    let column3_option4 = format!("{:<25}", "[7] Generator");

    println!("\n\n{}{}", header1, header3);
    println!("{}{}", column1_option1,  column3_option2);
    println!("{}{}", column1_option2, column3_option3);
    println!("{}{}", column1_option3, column3_option4);
    println!("{}{}", column1_option4, column3_option1);
}