pub mod commands;
pub mod core;

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
            commands::add::add().await;
        },
        "audit" => {
            // commands::audit::  
        },
        "search" => {
            commands::search::search();
        },
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
}
