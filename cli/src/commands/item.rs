use std::time::Duration;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize)]
struct Item {
    id: u32,
    title: String,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - ID: {}", self.title, self.id)
    }
}

pub async fn get_all(base: &str) {
    let url = format!("{base}/item");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client");

    let resp_res = client.get(url).send().await;

    if let Err(error) = resp_res {
        eprintln!("Request to get items failed: {error}");
        return;
    }

    let resp = resp_res.unwrap();
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        eprintln!("Error: HTTP {status} - {body}");
        return;
    }

    match resp.json::<Vec<Item>>().await {
        Ok(items) => {
            for item in &items {
                eprintln!("{}", item);
            }
            eprintln!("Found {} items", items.len());
        },
        Err(err) => {
            eprintln!("Error parsing data: {err}");
        }
    }


}