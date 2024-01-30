use std::error::Error;
use reqwest::Proxy;
use csv::StringRecord;
use serde_json;
use std::fs::File;
use std::io::BufRead;
use reqwest::header::{HeaderMap, HeaderValue};
use scraper::{Html, Selector};
use reqwest::Client;

struct ProductInfo {
    product_name: String,
    taille: String,
    instagram_name: String,
    email: String,
    first_name: String,
    last_name: String,
    address1: String,
    address2: String,
    city: String,
    country: String,
    province: String,
    zip: String,
    phone: String,
}

impl ProductInfo {
    fn from_csv_record(record: &StringRecord) -> Result<Self, Box<dyn Error>> {
        if record.len() < 13 {
            return Err(format!("not enough fields in record: {:?}", record).into());
        }

        Ok(Self {
            product_name: record[0].to_string(),
            taille: record[1].to_string(),
            instagram_name: record[2].to_string(),
            email: record[3].to_string(),
            first_name: record[4].to_string(),
            last_name: record[5].to_string(),
            address1: record[6].to_string(),
            address2: record[7].to_string(),
            city: record[8].to_string(),
            country: record[9].to_string(),
            province: record[10].to_string(),
            zip: record[11].to_string(),
            phone: record[12].to_string(),
        })
    }
}

async fn fetch_product_id(client: &Client, product_url: &str, taille: &str) -> Result<i64, Box<dyn Error>> {

    let product_name = product_url.rsplit('/').next().ok_or("Invalid URL format")?;

    let new_url = format!("https://raffles.afew-store.com/products/{}.json", product_name);

    let headers = build_request_headers();

    let response = client.get(&new_url)
                         .headers(headers)
                         .send()
                         .await?;

    let product_data: serde_json::Value = response.json().await?;

    let variants = product_data["product"]["variants"].as_array()
        .ok_or("No variants found")?;

    for variant in variants {
        if let Some(title) = variant["title"].as_str() {
            if title == taille {
                let product_id = variant["id"].as_i64()
                    .ok_or("Invalid ID format")?;
                
                return Ok(product_id);
            }
        }
    }

    Err("Size not found".into())
}

fn format_proxy_url(proxy: &str) -> Result<String, Box<dyn Error>> {
    let parts: Vec<&str> = proxy.split(":").collect();
    if parts.len() != 4 {
        return Err(format!("invalid proxy format: {}", proxy).into());
    }

    let (host, port, user, password) = (parts[0], parts[1], parts[2], parts[3]);
    Ok(format!("http://{}:{}@{}:{}", user, password, host, port))
}

use dialoguer::MultiSelect;

pub fn read_proxies() -> std::io::Result<Vec<String>> {
    let files = std::fs::read_dir(".")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().ok().map(|t| t.is_file()).unwrap_or(false))
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".txt"))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    if files.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No TXT files found in the current directory",
        ));
    }

    let selected_files = MultiSelect::new()
        .with_prompt("Select proxies file(s) | PRESS SPACE TO SELECT AND ENTER TO CONFIRM")
        .items(&files)
        .interact()?;

    if selected_files.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No proxies files selected",
        ));
    }

    let selected_files_paths: Vec<_> = selected_files
        .into_iter()
        .map(|index| std::path::Path::new(&files[index]))
        .collect();

    let mut proxies = Vec::new();

    for file_path in selected_files_paths {
        let file = std::fs::File::open(&file_path)?;
        let reader = std::io::BufReader::new(file);
        proxies.extend(reader.lines().filter_map(std::result::Result::ok));
    }

    Ok(proxies)
}

fn build_request_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Sec-Ch-Ua", HeaderValue::from_static("\"Google Chrome\";v=\"113\", \"Chromium\";v=\"113\", \"Not-A.Brand\";v=\"24\""));
    headers.insert("Sec-Ch-Ua-Mobile", HeaderValue::from_static("?0"));
    headers.insert("Sec-Ch-Ua-Platform", HeaderValue::from_static("Windows"));
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36"));
    headers.insert("Accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-site"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
    headers.insert("Sec-Fetch-User", HeaderValue::from_static("?1"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
    headers.insert("Referer", HeaderValue::from_static("https://en.afew-store.com/"));
    headers.insert("Accept-Language", HeaderValue::from_static("fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7,de;q=0.6"));
    headers.insert("Connection", HeaderValue::from_static("close"));
    headers
}

use chrono::Local;

async fn make_request(_api_url: &str, product_info: &ProductInfo, proxy: &str, task_index: usize) -> Result<(String, String, String, String), Box<dyn Error>> {
    println!("[{}] [Task {}] Entering data ...", Local::now().format("%Y-%m-%d %H:%M:%S"), task_index);

    let proxy_url = format_proxy_url(proxy)?;

    let request_headers = build_request_headers();

    let client = match reqwest::Client::builder()
        .proxy(Proxy::all(&proxy_url)?)
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error while building reqwest client: {:?}", e);
            return Err(e.into());
        }
    };

    let client_no_proxy = reqwest::Client::new();
    let id = fetch_product_id(
        &client_no_proxy,
        
        &product_info.product_name,
        &product_info.taille,
    )
    .await?;

    let url = format!(
        "https://raffles.afew-store.com/cart/{}:1?locale=en&attributes%5Blocale%5D=en&attributes%5Binstagram%5D={}&utm_source=",
        id,
        product_info.instagram_name
    );

    let response = match client
    .post(&url)
    .headers(request_headers)
    .send()
    .await
{
    Ok(response) => response,
    Err(e) => {
        eprintln!("Error while sending request: {:?}", e);
        return Err(e.into());
    }
};

 // println!("URL: {}", response.url());


let cookies = response.cookies()
    .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
    .collect::<Vec<String>>();


// println!("Cookies: {:?}", cookies);


let response_url = response.url().to_string();

// println!("{}", response_url);


let response_text = match response.text().await {
    Ok(text) => text,
    Err(e) => {
        // eprintln!("Error while reading response text: {:?}", e);
        return Err(e.into());
    }
};

if response_text.is_empty() {
    // eprintln!("Response body is empty.");
} else {
    let document = Html::parse_document(&response_text);
    let authenticity_token_selector =
        Selector::parse("input[name='authenticity_token']").unwrap();
    let og_image_selector = Selector::parse("meta[property='og:image']").unwrap();

    let authenticity_token = document
        .select(&authenticity_token_selector)
        .next()
        .map(|element| element.value().attr("value").unwrap_or(""));
    let og_image_content = document
        .select(&og_image_selector)
        .next()
        .map(|element| element.value().attr("content").unwrap_or(""));

    if let Some(token) = authenticity_token {
        //  println!("authenticity_token: {}", token);
    }
    if let Some(image) = og_image_content {
        //  println!("og:image: {}", image);
    }

   
    return Ok((cookies.join("; "), authenticity_token.unwrap_or("").to_string(), response_url, og_image_content.unwrap_or("").to_string()));
}


Ok((String::new(), String::new(), String::new(), String::new()))
}

async fn make_second_request(
    product_info: &ProductInfo, 
    proxy: &str, 
    cookies: String, 
    authenticity_token: String, 
    response_url: String
) -> Result<(String, String, String ), Box<dyn Error>> {

  
    let proxy_url = format_proxy_url(proxy)?;

    let mut request_headers = build_request_headers();
    request_headers.insert("Content-Type", HeaderValue::from_static("application/x-www-form-urlencoded"));
    request_headers.insert("Cookie", HeaderValue::from_str(&cookies)?);

    let client = match reqwest::Client::builder()
    .proxy(Proxy::all(&proxy_url)?)
    .redirect(reqwest::redirect::Policy::none()) 
    .build()
{
    Ok(client) => client,
    Err(e) => {
        eprintln!("Error while building reqwest client: {:?}", e);
        return Err(e.into());
    }
};

    let parameters = vec![
        ("_method", "patch"),
        ("authenticity_token", &authenticity_token),
        ("previous_step", "contact_information"),
        ("step", "shipping_method"),
        ("checkout[email]", &product_info.email),
        ("checkout[attributes][locale]", "en"),
        ("checkout[attributes][instagram]", &product_info.instagram_name),
        ("checkout[shipping_address][first_name]", &product_info.first_name),
        ("checkout[shipping_address][last_name]", &product_info.last_name),
        ("checkout[shipping_address][company]", ""),
        ("checkout[shipping_address][address1]", &product_info.address1),
        ("checkout[shipping_address][address2]", &product_info.address2),
        ("checkout[shipping_address][city]", &product_info.city),
        ("checkout[shipping_address][country]", &product_info.country),
        ("checkout[shipping_address][province]", &product_info.province),
        ("checkout[shipping_address][zip]", &product_info.zip),
        ("checkout[shipping_address][phone]", &product_info.phone),
        ("checkout[remember_me]", "0"),
        ("checkout[client_details][browser_width]", "1263"),
        ("checkout[client_details][browser_height]", "577"),
        ("checkout[client_details][javascript_enabled]", "1"),
        ("checkout[client_details][color_depth]", "24"),
        ("checkout[client_details][java_enabled]", "false"),
        ("checkout[client_details][browser_tz]", "-120")
    ];

    let response = match client
        .post(&response_url)
        .headers(request_headers)
        .form(&parameters)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Error while sending request 2 : {:?}", e);
            return Err(e.into());
        }
    };

    
    let new_cookies = response.cookies()
        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
        .collect::<Vec<String>>();

        let response_url2 = response.url().to_string();
   
    

   
        let response_text = match response.text().await {
            Ok(text) => {
                // println!("Response body: {}", text);
                text
            },
            Err(e) => {
                // eprintln!("Error while reading response text: {:?}", e);
                return Err(e.into());
            }
        };
    let document = Html::parse_document(&response_text);
    let authenticity_token_selector2 = Selector::parse("input[name='authenticity_token']").unwrap();

    let new_authenticity_token = document
        .select(&authenticity_token_selector2)
        .next()
        .map(|element| element.value().attr("value").unwrap_or(""));

    // println!("New authenticity_token: {}", new_authenticity_token.unwrap_or(""));
     // println!("URL2 {}", response_url2);
   
    return Ok((new_cookies.join("; "), new_authenticity_token.unwrap_or("").to_string(), response_url2));
}

async fn make_third_request(
    proxy: &str, 
    new_cookies: String, 
    new_authenticity_token: String, 
    response_url2: String
) -> Result<(String, String, String, String ), Box<dyn Error>> {

  
    let proxy_url = format_proxy_url(proxy)?;

    let mut request_headers = build_request_headers();
    request_headers.insert("Content-Type", HeaderValue::from_static("application/x-www-form-urlencoded"));
    request_headers.insert("Cookie", HeaderValue::from_str(&new_cookies)?);

    let client = match reqwest::Client::builder()
    .proxy(Proxy::all(&proxy_url)?)
     // .redirect(reqwest::redirect::Policy::none()) 
    .build()
{
    Ok(client) => client,
    Err(e) => {
        eprintln!("Error while building reqwest client 2 : {:?}", e);
        return Err(e.into());
    }
};

let parameters = vec![
    ("_method", "patch"),
    ("authenticity_token", &new_authenticity_token),
    ("previous_step", "shipping_method"),
    ("step", "payment_method"),
    ("checkout[shipping_rate][id]", "shopify-DHL-7.99"), // Check here before because change often
    ("checkout[client_details][browser_width]", "1263"),
    ("checkout[client_details][browser_height]", "577"),
    ("checkout[client_details][javascript_enabled]", "1"),
    ("checkout[client_details][color_depth]", "30"),
    ("checkout[client_details][java_enabled]", "false"),
    ("checkout[client_details][browser_tz]", "-120")
];

    let response = match client
        .post(&response_url2)
        .headers(request_headers)
        .form(&parameters)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Error while sending request 3 : {:?}", e);
            return Err(e.into());
        }
    };

    
    let new_cookies2 = response.cookies()
        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
        .collect::<Vec<String>>();

    let response_url3 = response.url().to_string();

    // println!("URL 3 : {}", response_url3);

    let response_text = match response.text().await {
        Ok(text) => {
         // println!("Response body: {}", text); 
            text
        },
        Err(e) => {
            // eprintln!("Error while reading response text: {:?}", e);
            return Err(e.into());
        }
    };

    let document = Html::parse_document(&response_text);
    let payment_subform_selector = Selector::parse("div[data-payment-subform='required']").unwrap();
    let radio_wrapper_selector = Selector::parse("div.radio-wrapper").unwrap();
    let mut gateway_value = String::new();

    for payment_subform_element in document.select(&payment_subform_selector) {
        for radio_wrapper_element in payment_subform_element.select(&radio_wrapper_selector) {
            if let Some(value) = radio_wrapper_element.value().attr("data-select-gateway") {
                gateway_value = value.to_string();
            }
        }
    }

     // println!("Gateway Value: {}", gateway_value);

    // println!("body {}", response_text);
    
    let authenticity_token_selector2 = Selector::parse("input[name='authenticity_token']").unwrap();

    let new_authenticity_token2 = document
        .select(&authenticity_token_selector2)
        .next()
        .map(|element| element.value().attr("value").unwrap_or(""));

     // println!("New authenticity_token: {}", new_authenticity_token2.unwrap_or(""));
    //  println!("URL2 {}", response_url3);
   
    return Ok((new_cookies2.join("; "), new_authenticity_token2.unwrap_or("").to_string(), response_url2, gateway_value));
}

async fn make_fourth_request(
    proxy: &str, 
    new_cookies2: String, 
    new_authenticity_token2: String, 
    response_url3: String,
    gateway_value: String,
) -> Result<(String, String, String, String ), Box<dyn Error>> {

    let proxy_url = format_proxy_url(proxy)?;

    let mut request_headers = build_request_headers();
    request_headers.insert("Content-Type", HeaderValue::from_static("application/x-www-form-urlencoded"));
    request_headers.insert("Cookie", HeaderValue::from_str(&new_cookies2)?);

    let client = match reqwest::Client::builder()
        .proxy(Proxy::all(&proxy_url)?)
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error while building reqwest client 4 : {:?}", e);
            return Err(e.into());
        }
    };

    let parameters = vec![
        ("_method", "patch"),
        ("authenticity_token", &new_authenticity_token2),
        ("previous_step", "payment_method"),
        ("step", "review"),
        ("checkout[payment_gateway]", &gateway_value),
        ("checkout[different_billing_address]", "false"),
        ("checkout[client_details][browser_width]", "1263"),
        ("checkout[client_details][browser_height]", "577"),
        ("checkout[client_details][javascript_enabled]", "1"),
        ("checkout[client_details][color_depth]", "24"),
        ("checkout[client_details][java_enabled]", "false"),
        ("checkout[client_details][browser_tz]", "-120"),
        
    ];

    let response = match client
        .post(&response_url3)
        .headers(request_headers)
        .form(&parameters)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Error while sending request 4: {:?}", e);
            return Err(e.into());
        }
    };

    let new_cookies3 = response.cookies()
        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
        .collect::<Vec<String>>();
   
    let response_url4 = response.url().to_string();

    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            // eprintln!("Error while reading response text: {:?}", e);
            return Err(e.into());
        }
    };

    let document = Html::parse_document(&response_text);
    
    let price_selector = Selector::parse("input[name='checkout[total_price]']").unwrap();

    let price = document
        .select(&price_selector)
        .next()
        .map(|element| element.value().attr("value").unwrap_or(""));

    // println!("New price: {}", price.unwrap_or(""));

    // println!("body {}", response_text);
    let authenticity_token_selector3 = Selector::parse("input[name='authenticity_token']").unwrap();

    let new_authenticity_token3 = document
        .select(&authenticity_token_selector3)
        .next()
        .map(|element| element.value().attr("value").unwrap_or(""));

    // println!("New authenticity_token: {}", new_authenticity_token3.unwrap_or(""));

    // println!("URL4 {}", response_url4);
   
    return Ok((new_cookies3.join("; "), new_authenticity_token3.unwrap_or("").to_string(), response_url4, price.unwrap_or("").to_string()));
}

async fn make_fifth_request(
    proxy: &str, 
    new_cookies3: String, 
    new_authenticity_token3: String, 
    response_url4: String,
    price: String,
    task_index: usize,
) -> Result<(String, String, String ), Box<dyn Error>> {

    let proxy_url = format_proxy_url(proxy)?;

    let mut request_headers = build_request_headers();
    request_headers.insert("Content-Type", HeaderValue::from_static("application/x-www-form-urlencoded"));
    request_headers.insert("Cookie", HeaderValue::from_str(&new_cookies3)?);

    let client = match reqwest::Client::builder()
        .proxy(Proxy::all(&proxy_url)?)
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error while building reqwest client 5 : {:?}", e);
            return Err(e.into());
        }
    };

    let parameters = vec![
        ("_method", "patch"),
        ("authenticity_token", &new_authenticity_token3),
        ("complete", "1"),
        ("checkout[total_price]", &price),
        ("checkout[client_details][browser_width]", "1263"),
        ("checkout[client_details][browser_height]", "577"),
        ("checkout[client_details][javascript_enabled]", "1"),
        ("checkout[client_details][color_depth]", "24"),
        ("checkout[client_details][java_enabled]", "false"),
        ("checkout[client_details][browser_tz]", "-120"),
    ];

    let response = match client
        .post(&response_url4)
        .headers(request_headers)
        .form(&parameters)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Error while sending request 5 : {:?}", e);
            return Err(e.into());
        }
    };

    let new_cookies4 = response.cookies()
        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
        .collect::<Vec<String>>();
   
    let response_url5 = response.url().to_string();

    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            // eprintln!("Error while reading response text: {:?}", e);
            return Err(e.into());
        }
    };

    let document = Html::parse_document(&response_text);

    // println!("body {}", response_text);
    let authenticity_token_selector4 = Selector::parse("input[name='authenticity_token']").unwrap();

    let new_authenticity_token4 = document
        .select(&authenticity_token_selector4)
        .next()
        .map(|element| element.value().attr("value").unwrap_or(""));

    // println!("New authenticity_token: {}", new_authenticity_token4.unwrap_or(""));

    // println!("URL5 {}", response_url5);
    println!("[{}] [Task {}] Entry successful ...", Local::now().format("%Y-%m-%d %H:%M:%S"), task_index);

    return Ok((new_cookies4.join("; "), new_authenticity_token4.unwrap_or("").to_string(), response_url5));
}

use std::time::Duration;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    delay: u64,
    discord_webhook_url: String,

}

use serde_json::json;

async fn send_discord_webhook(
    webhook_url: &str,
    email: &str,
    product_name: &str,
    size: &str,
    og_image_content: &str,
    color: u32,
) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let payload = json!({
        "username": "FluxyIO",
        "avatar_url": "https://image.noelshack.com/fichiers/2023/35/2/1693332455-image-3.png",
        "embeds": [{
            "title": format!("Raffle Entry success"),
            "fields": [
                {
                    "name": "Website",
                    "value": "||Afew||",
                    "inline": false
                },
                {
                    "name": "Email",
                    "value": email,
                    "inline": false
                },
                {
                    "name": "Product",
                    "value": product_name,
                    "inline": false
                },
                {
                    "name": "Size",
                    "value": size,
                    "inline": false
                }
            ],
            "footer": {
                "text": "FluxyIO Raffle",
                "icon_url": "https://image.noelshack.com/fichiers/2023/35/2/1693332455-image-3.png"
            },
            "thumbnail": {
                "url": og_image_content
            },
            "color": color
        }]
    });
    let response = client.post(webhook_url).json(&payload).send().await?;
    Ok(())
}

async fn send_discord_webhook2(
    webhook_url2: &str,
    email: &str,
    product_name: &str,
    size: &str,
    og_image_content: &str,
    color: u32,
) -> Result<(), Box<dyn Error>> {
    let client2 = reqwest::Client::new();
    let payload2 = json!({
        "username": "FluxyIO",
        "avatar_url": "https://image.noelshack.com/fichiers/2023/35/2/1693332455-image-3.png",
        "embeds": [{
            "title": format!("Raffle Entry success"),
            "fields": [
                {
                    "name": "Website",
                    "value": "||Afew||",
                    "inline": false
                },
                {
                    "name": "Product", 
                    "value": product_name,
                    "inline": false
                },
                {
                    "name": "Size",
                    "value": size,
                    "inline": false
                }
            ],
            "footer": {
                "text": "FluxyIO Raffle",
                "icon_url": "https://image.noelshack.com/fichiers/2023/35/2/1693332455-image-3.png"
            },
            "thumbnail": {
                "url": og_image_content
            },
            "color": color
        }]
    });
    let response = client2.post(webhook_url2).json(&payload2).send().await?;
    Ok(())
}




pub async fn process_csv(api_url: &str, csv_file_path: &str) -> Result<usize, Box<dyn Error>> {
    let mut csv_reader = csv::Reader::from_path(csv_file_path)?;
    let total_records = csv_reader.records().count();
    let mut csv_reader = csv::Reader::from_path(csv_file_path)?;
    let proxies = read_proxies().unwrap();

    let config: Config = serde_json::from_reader(File::open("config.json")?)?;

    for (index, result) in csv_reader.records().enumerate() {
        if index >= proxies.len() {
            break; 
        }

        let record = result?;
        let product_info = match ProductInfo::from_csv_record(&record) {
            Ok(info) => info,
            Err(e) => {
                // eprintln!("Error while creating ProductInfo from CSV record: {:?}", e);
                return Err(e.into());
            }
        };

        let proxy = &proxies[index];
        

        let (cookies, authenticity_token, response_url, og_image_content) = match make_request(api_url, &product_info, proxy, index).await {
            Ok(results) => results,
            Err(e) => {
                // eprintln!("Error while making request: {:?}", e);
                continue; 
            }
        };

        let (new_cookies, new_authenticity_token, response_url2 ) = match make_second_request(&product_info, proxy, cookies, authenticity_token, response_url).await {
            Ok(results) => results,
            Err(e) => {
                // eprintln!("Error while making second request: {:?}", e);
                continue;
            }
        };

        let (new_cookies2, new_authenticity_token2, response_url2, gateway_value ) = match make_third_request( proxy, new_cookies, new_authenticity_token, response_url2).await {
            Ok(results) => results,
            Err(e) => {
                // eprintln!("Error while making second request: {:?}", e);
                continue;
            }
        };

        let (new_cookies3, new_authenticity_token3, response_url3, price ) = match make_fourth_request( proxy, new_cookies2, new_authenticity_token2, response_url2, gateway_value).await {
            Ok(results) => results,
            Err(e) => {
                // eprintln!("Error while making second request: {:?}", e);
                continue;
            }
        };

        if let Err(e) = make_fifth_request(proxy, new_cookies3, new_authenticity_token3, response_url3, price, index).await {
            // eprintln!("Error while making third request: {:?}", e);
        }

        
        let webhook_url = &config.discord_webhook_url;
        let color = 16711680; 
        if let Err(e) = send_discord_webhook(webhook_url, &product_info.email, &product_info.product_name, &product_info.taille, &og_image_content, color).await {
            eprintln!("Error while sending Discord webhook: {:?}", e);
        }

       // let webhook_url2 = "https://discord.com/api/webhooks/1109895813502607400/nomNh3BxTh00Q-6BhxB2pAm49k3JUL8K6xZyl9hOWOC4Ki3DjxneMGFxm4cs0OSBKNZg";
       // let color = 16711680; 
       // if let Err(e) = send_discord_webhook2(webhook_url2, &product_info.email, &product_info.product_name, &product_info.taille, &og_image_content, color).await {
         //   eprintln!("Error while sending Discord webhook: {:?}", e);
       // }

        if index < total_records - 1 {
            tokio::time::sleep(Duration::from_secs(config.delay)).await;
        }

        
    }

    Ok(total_records)
}