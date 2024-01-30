use std::fs::File;
use std::io::prelude::*;
use serde_json::json;

pub fn is_first_launch() -> bool {
    !file_exists("config.json") || !file_exists("data.csv")
}

pub fn create_files_with_content() {
    if !file_exists("config.json") {
        create_file_with_content("config.json", &json!({
            "api_key": "",
            "delay": 60,
            "discord_webhook_url": "",
            "email": "",
            "password": ""
        }));
    }

    if !file_exists("data.csv") {
        create_data_file_with_content("data.csv", "ProductName,Taille,InstagramName,Email,FirstName,LastName,Address1,Address2,City,Country,Province,Zip,Phone\nAir%20Jordan%204%20Retro%20%27Thunder%27,,,exemple@gmail.com,Bernard,ARNAUD,1 champ elysé,,Paris,FR,Ile-De-France,75000,0606060606");
    }
}


fn file_exists(filename: &str) -> bool {
    std::path::Path::new(filename).exists()
}

fn create_file_with_content(filename: &str, content: &serde_json::Value) {
    match File::create(filename) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(content.to_string().as_bytes()) {
                println!("Erreur lors de l'écriture du contenu dans {}: {}", filename, e);
            } else {
                println!("Le fichier {} a été créé avec succès.", filename);
            }
        }
        Err(e) => println!("Erreur lors de la création du fichier {}: {}", filename, e),
    }
}

fn create_data_file_with_content(filename: &str, content: &str) {
    match File::create(filename) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(content.as_bytes()) {
                println!("Erreur lors de l'écriture du contenu dans {}: {}", filename, e);
            } else {
                println!("Le fichier {} a été créé avec succès.", filename);
            }
        }
        Err(e) => println!("Erreur lors de la création du fichier {}: {}", filename, e),
    }
}
