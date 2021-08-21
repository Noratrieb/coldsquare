use coldsquare::{display_class, parse_class_file};

fn main() {
    let file = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("No file provided");
        std::process::exit(1);
    });
    let file = std::fs::read(file).unwrap_or_else(|_| {
        eprintln!("Could not read file");
        std::process::exit(1);
    });

    let class_file = match parse_class_file(&file) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    let stdout = std::io::stdout();

    if let Err(why) = display_class(stdout.lock(), &class_file) {
        eprintln!("{}", why);
    }
}
