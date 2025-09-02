use std::collections::HashMap;

use crate::core::cli::handle_http_error;


pub async fn add() -> Result<(), reqwest::Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        let app_name = args.get(0)
            .expect("Failed to get App Name");
        eprintln!("{} add <item_name> [options]", app_name);
        std::process::exit(1);
    }

    let response = reqwest::get("http://localhost:8082/add")
        .await;

    if response.is_err() {
        let t = response.err().unwrap();
        handle_http_error(&t);
    }

    let resp = reqwest::get("http://localhost:8082/add")
        .await?
        .text()
        // .json::<HashMap<String, String>>()
        .await?;

    eprintln!("Response: {}", resp);
    
    Ok(())
}