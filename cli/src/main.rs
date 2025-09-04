pub mod commands;
pub mod core;

const BASE_URL: &str = "http://localhost:8082";

#[tokio::main]
async fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        commands::help::help();
        std::process::exit(1);
    }

    let command = args.get(1).unwrap();
    
    match command.as_str() {
        "add" => {
            commands::add::add(BASE_URL).await;
        },
        "audit" => {
            // commands::audit::  
        },
        "search" => {
            commands::search::search();
        },
        "item" => {
            commands::item::get_all(BASE_URL).await;
        },
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
}
