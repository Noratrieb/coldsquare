fn main() {
    let file = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("No file provided");
        std::process::exit(1);
    });

    let contents = std::fs::read(file).unwrap_or_else(|_| {
        eprintln!("Could not read file");
        std::process::exit(1);
    });

    let class_file = match cs_parser::parse_class_file(&contents) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    cs_class_printer::print(&class_file);
}
