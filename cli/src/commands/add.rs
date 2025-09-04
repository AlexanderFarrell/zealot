use std::{collections::HashMap, time::Duration};
use serde::Serialize;

use crate::core::cli::handle_http_error;

#[derive(Serialize)]
struct AddInfo {
    title: String,
}

pub async fn add(base: &str) {
    let url = format!("{base}/item");

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        let app_name = args.get(0)
            .expect("Failed to get App title");
        eprintln!("{} add <item_title> [options]", app_name);
        std::process::exit(1);
    }

    let item_title = &args[2];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Faild to build HTTP client");

    let payload = AddInfo {
        title: item_title.to_string(),
    };

    match client.post(url).json(&payload).send().await {
        Ok(resp) if resp.status().is_success() => {
            eprintln!("✅ Added \"{}\"", item_title);
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            eprintln!("Error: HTTP {status} - {body}");
        }
        Err(err) => {
            eprintln!("Request to add item failed: {err}");
        }
    }
}