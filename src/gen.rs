use std::fs::File;
use std::io::prelude::*;
use std::io;
use rand::Rng;

pub fn read_input(prompt: &str) -> String {
    let mut input = String::new();
    println!("{}", prompt);
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn generate_phone_number(country_code: &str) -> Option<String> {
    let mut rng = rand::thread_rng();
    let number = match country_code {
        "FR" => Some(format!("0{}{}", rng.gen_range(6..8), rng.gen_range(10000000..100000000))),
        "US" => Some(format!("{}{}{}", rng.gen_range(100..1000), rng.gen_range(100..1000), rng.gen_range(1000..10000))),
        "DE" => Some(format!("0{}{}", rng.gen_range(15..18), rng.gen_range(1000000..10000000))),
        "UK" => Some(format!("07{}", rng.gen_range(100000000..1000000000))),
        "CA" => Some(format!("{}{}{}", rng.gen_range(100..1000), rng.gen_range(100..1000), rng.gen_range(1000..10000))),
        "IT" => Some(format!("3{}{}", rng.gen_range(2..4), rng.gen_range(100000000..1000000000))),
        "ES" => Some(format!("{}{}", rng.gen_range(6..8), rng.gen_range(100000000..1000000000))),
        "AU" => Some(format!("04{}", rng.gen_range(10000000..100000000))),
        _ => None
    };
    number
}

pub fn generate_phone_numbers(country_code: &str, num_of_times: u32) -> Option<String> {
    let mut contents = String::new();
    for _ in 0..num_of_times {
        match generate_phone_number(&country_code) {
            Some(number) => {
                contents.push_str(&number);
                contents.push_str("\n");
            },
            None => {
                println!("Unsupported country code.");
                return None;
            }
        }
    }
    Some(contents)
}

pub fn write_to_file(filename: String, contents: String) {
    let mut file = File::create(&filename).expect("Unable to create file");
    file.write_all(contents.as_bytes()).expect("Unable to write to file");
    println!("File saved as {}", filename);
}


