fn main() {
    if let Err(error) = bolt::run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
