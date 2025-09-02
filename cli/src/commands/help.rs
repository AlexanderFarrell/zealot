pub fn help() {
    let args: Vec<String> = std::env::args().collect();
    let prog_name = args.get(0);

    match prog_name {
        None => panic!("No program name"),
        Some(name) => {
            eprintln!("Usage: {} <command> [options]", name)
        }
    }
}