mod model;
mod services;
mod secure;
mod afew;
mod ppgrabber;
mod jig;
mod gen;
mod lunch;

#[macro_use]
extern crate serde_derive;

extern crate reqwest;
extern crate serde_json;
extern crate winapi;

use std::io::{self};
use console::Term;
use tokio::time::Duration;
use crossterm::{style::{Color, Print, ResetColor, SetForegroundColor}, execute};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task;


async fn display_menu_loop(username: Arc<String>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut stdout = io::stdout();
        let stdin = io::stdin();
        let term = Term::stdout();

        let ascii_art = r#"
         ______ _                  _____ ____    _____        __  __ _      
        |  ____| |                |_   _/ __ \  |  __ \      / _|/ _| |     
        | |__  | |_   ___  ___   _  | || |  | | | |__) |__ _| |_| |_| | ___ 
        |  __| | | | | \ \/ / | | | | || |  | | |  _  // _` |  _|  _| |/ _ \
        | |    | | |_| |>  <| |_| |_| || |__| | | | \ \ (_| | | | | | |  __/
        |_|    |_|\__,_/_/\_\\__, |_____\____/  |_|  \_\__,_|_| |_| |_|\___|
                              __/ |                                         
                             |___/                                                                   
                                                             
        "#;

        execute!(stdout, SetForegroundColor(Color::DarkRed), Print(ascii_art), ResetColor)?;

        let username_box: Box<dyn std::fmt::Display> = Box::new(username.clone());

        let api_url = "http://34.163.228.37:8080";
        let csv_file_path = "data.csv";

        println!("\nSuccessfully logged in as: {}", &*username_box);

        model::display_menu();
        execute!(stdout, SetForegroundColor(Color::White), Print("\n\nWhat would you like to do next?\n"), ResetColor)?;

        let mut choice = String::new();
        stdin.read_line(&mut choice)?;
        let choice: i32 = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                execute!(stdout, SetForegroundColor(Color::Red), Print("\n\nInvalid choice. Please enter a valid number.\n"), ResetColor)?;
                continue;
            }
        };

        match choice {
            1 => {
                term.clear_screen().unwrap();
                execute!(stdout, SetForegroundColor(Color::Yellow), Print("\n\nRunning task for Afew Raffle...\n"), ResetColor)?;

                let task_count = match afew::process_csv(api_url, csv_file_path).await {
                    Ok(count) => {
                        execute!(stdout, SetForegroundColor(Color::Green), Print("\nTask completed successfully!\n"), ResetColor)?;
                        count
                    },
                    Err(e) => {
                        execute!(stdout, SetForegroundColor(Color::Red), Print(format!("\nAn error occurred: {}\n", e)), ResetColor)?;
                        0
                    }
                };


                execute!(stdout, SetForegroundColor(Color::Green), Print("Press Enter to Continue"), ResetColor)?;
                let _ = stdin.read_line(&mut String::new());
                term.clear_screen().unwrap();
            },
            
            8 => {
                execute!(stdout, SetForegroundColor(Color::Yellow), Print("Exiting the program..."), ResetColor)?;
                std::process::exit(0);
            },
            5 => {
                term.clear_screen().unwrap();
                execute!(stdout, SetForegroundColor(Color::Yellow), Print("\n\nJIG\n"), ResetColor)?;
                let mut address = String::new();
                let mut mode = String::new();
                let mut filename = String::new();
                let mut num_times = String::new();

                execute!(stdout, SetForegroundColor(Color::White), Print("Enter your address: "), ResetColor)?;
                std::io::stdin().read_line(&mut address).unwrap();

                execute!(stdout, SetForegroundColor(Color::White), Print("Enter jig mode (soft, medium, hard, special): "), ResetColor)?;
                std::io::stdin().read_line(&mut mode).unwrap();

                execute!(stdout, SetForegroundColor(Color::White), Print("Enter filename: "), ResetColor)?;
                std::io::stdin().read_line(&mut filename).unwrap();

                execute!(stdout, SetForegroundColor(Color::White), Print("Enter the number of times to generate the address: "), ResetColor)?;
                std::io::stdin().read_line(&mut num_times).unwrap();
                let num_times: usize = num_times.trim().parse().unwrap();

                match jig::jig(&address.trim(), &mode.trim(), &filename.trim(), num_times) {
                    Ok(_) => execute!(stdout, SetForegroundColor(Color::Green), Print("File successfully created\n"), ResetColor)?,
                    Err(_) => execute!(stdout, SetForegroundColor(Color::Red), Print("Error while creating the file\n"), ResetColor)?,
                }

                execute!(stdout, SetForegroundColor(Color::Green), Print("Press Enter to Continue"), ResetColor)?;
                let _ = stdin.read_line(&mut String::new());
                term.clear_screen().unwrap();
            },
            6 => {
                term.clear_screen().unwrap();
                execute!(stdout, SetForegroundColor(Color::Yellow), Print("\n\nRunning PayPal Link Grabber...\n\n"), ResetColor)?;

                let sender = "noreply@afew-store.com";
                let mut seen = HashSet::new();
                let webhook_url = "";

                let should_stop = Arc::new(AtomicBool::new(false));

                let should_stop_for_task = Arc::clone(&should_stop);

                let wait_for_enter_task = task::spawn(async move {
                    let mut line = String::new();
                    io::stdin().read_line(&mut line).unwrap();
                    should_stop_for_task.store(true, Ordering::Relaxed);
                });

                tokio::spawn(async move {
                    ppgrabber_main(sender, webhook_url, &mut seen, should_stop).await;
                });

                execute!(stdout, SetForegroundColor(Color::Green), Print("Press Enter to Continue/ Don't Press Continue if you want to continue to grab pp link\n"), ResetColor)?;

                wait_for_enter_task.await.unwrap();
                term.clear_screen().unwrap();
            },
            7 => {
                term.clear_screen().unwrap();
                execute!(stdout, SetForegroundColor(Color::Yellow), Print("Select an option..."), ResetColor)?;
            
                println!("\n1 => Number");
                println!("2 => Name");
                println!("3 => Surname");
            
                let mut selection = String::new();
                stdin.read_line(&mut selection)?;
                let selection: i32 = match selection.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        execute!(stdout, SetForegroundColor(Color::Red), Print("\n\nInvalid choice. Please enter a valid number.\n"), ResetColor)?;
                        continue;
                    }
                };
                term.clear_screen().unwrap();
            
                match selection {
                    1 => {
                        
                        execute!(stdout, SetForegroundColor(Color::Yellow), Print("You selected Number\n"), ResetColor)?;
                        let country_code = gen::read_input("Please enter country code:");
                        let num_of_times: u32 = gen::read_input("Please enter the number of phone numbers to generate:").parse().expect("Please enter a number!");
                        let filename = format!("{}.txt", gen::read_input("Please enter filename:").trim_end_matches(".txt"));

                        if let Some(contents) = gen::generate_phone_numbers(&country_code, num_of_times) {
                            gen::write_to_file(filename, contents);
                        }
                    },
                    2 => {
                        execute!(stdout, SetForegroundColor(Color::Yellow), Print("You selected Name"), ResetColor)?;
                        
                    },
                    3 => {
                        execute!(stdout, SetForegroundColor(Color::Yellow), Print("You selected Surname"), ResetColor)?;
                        
                    },
                    _ => {
                        execute!(stdout, SetForegroundColor(Color::Red), Print("\n\nInvalid choice. Please enter a valid number.\n"), ResetColor)?;
                        continue;
                    }
                }
            
                execute!(stdout, SetForegroundColor(Color::Green), Print("Press Enter to Continue"), ResetColor)?;
                let _ = stdin.read_line(&mut String::new());
                term.clear_screen().unwrap();
            },
            _ => {
                execute!(stdout, SetForegroundColor(Color::Red), Print("Invalid choice. Please enter a valid number."), ResetColor)?;
                continue;
            }
        }

    }
}

async fn ppgrabber_main(sender: &str, webhook_url: &str, seen: &mut HashSet<String>, should_stop: Arc<AtomicBool>) {
    loop {
        if should_stop.load(Ordering::Relaxed) {
            break;
        }
        let email_bodies = ppgrabber::fetch_emails_from_sender(sender, seen);
        match email_bodies {
            Ok(bodies) => {
                if bodies.is_empty() {
                    println!("No emails found from {} in the last 30 minutes.", sender);
                } else {
                    for body in bodies {
                        let links = ppgrabber::extract_links_from_email_body(&body);
                        for link in links {
                            if let Err(err) = ppgrabber::send_webhook_with_embed(webhook_url, &link).await {
                                println!("Failed to send webhook(too many links)");
                            }
                        }
                    }
                }
            }
            Err(e) => println!("An error occurred: {}", e),
        }

        tokio::time::sleep(Duration::from_secs(20)).await;
    }
}

use tokio::time;
use crate::secure::detect_sniffing_processes;
use crate::lunch::is_first_launch;
use crate::lunch::create_files_with_content;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    if is_first_launch() {
        create_files_with_content();
    }

    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let term = Term::stdout();

    let ascii_art = r#"
     ______ _                  _____ ____    _____        __  __ _      
    |  ____| |                |_   _/ __ \  |  __ \      / _|/ _| |     
    | |__  | |_   ___  ___   _  | || |  | | | |__) |__ _| |_| |_| | ___ 
    |  __| | | | | \ \/ / | | | | || |  | | |  _  // _` |  _|  _| |/ _ \
    | |    | | |_| |>  <| |_| |_| || |__| | | | \ \ (_| | | | | | |  __/
    |_|    |_|\__,_/_/\_\\__, |_____\____/  |_|  \_\__,_|_| |_| |_|\___|
                          __/ |                                         
                         |___/                                                                        
                                                          
    "#;

    execute!(stdout, SetForegroundColor(Color::DarkRed), Print(ascii_art), ResetColor)?;

    let api_key = services::load_api_key().await?;

    match services::check_login(&api_key).await {
        Ok((true, username)) => {
            let username_clone = username.clone();
            let username = Arc::new(username);

            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(60)); 
                loop {
                    interval.tick().await;
                    detect_sniffing_processes(username_clone.clone()).await;
                }
            });

            display_menu_loop(username).await?;
        },
        _ => {
            execute!(stdout, SetForegroundColor(Color::Red), Print("Invalid key/Metadata"), ResetColor)?;
            std::process::exit(1);
        }
    }

    Ok(())
}
