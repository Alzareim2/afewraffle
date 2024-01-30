// jig.rs

use std::fs::File;
use std::io::Write;
use rand::{Rng, SeedableRng};
use rand::distributions::Alphanumeric;
use rand::rngs::StdRng;

pub fn jig(address: &str, mode: &str, filename: &str, num_times: usize) -> std::io::Result<()> {
    let filename_with_extension = format!("{}.txt", filename);
    let mut file = File::create(filename_with_extension)?;

    for _ in 0..num_times {
        match mode {
            "soft" => file.write_all(soft(address).as_bytes())?,
            "medium" => file.write_all(medium(address).as_bytes())?,
            "hard" => file.write_all(hard(address).as_bytes())?,
            "special" => file.write_all(special(address).as_bytes())?,
            _ => panic!("Unknown jig mode"),
        }

        file.write_all(b"\n")?; 
    }

    Ok(())
}

fn soft(address: &str) -> String {
    let rng = StdRng::from_entropy(); 
    let letters: String = rng
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect();

    format!("{} {} {}", letters, address, letters)
}

fn medium(address: &str) -> String {
    let mut rng = StdRng::from_entropy();
    address
        .chars()
        .map(|c| {
            if c.is_ascii_digit() {
                c.to_string()
            } else {
                let variation: u8 = rng.gen_range(0..=2);
                let doubled_char: String = c.to_string().repeat(variation as usize + 1);
                doubled_char
            }
        })
        .collect()
}

fn hard(address: &str) -> String {
    let replacements = [
        ("rue", "chemin"),
        ("allée", "passage"),
        ("chemin", "voie")
    ];

    let mut new_address = address.to_string();

    for (old, new) in replacements.iter() {
        new_address = new_address.replace(old, new);
    }

    medium(&new_address)
}

fn special(address: &str) -> String {
    let mut rng = StdRng::from_entropy();
    address
        .chars()
        .map(|c| if c == 'X' {
            rng.gen_range('A'..='Z')
        } else {
            c
        })
        .collect()
}


