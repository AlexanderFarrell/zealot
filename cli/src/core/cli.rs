pub fn handle_http_error(response_error: &reqwest::Error) {
    eprintln!("Error Communicating with Zealot: {}", response_error);
}