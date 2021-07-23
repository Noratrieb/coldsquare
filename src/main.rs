use coldsquare::parse_class_file;

fn main() {
    let file = "primeNumberChecker.class";
    let file = std::fs::read(file).unwrap();

    let class_file = parse_class_file(file).unwrap();

    println!("{:?}", class_file);
}
