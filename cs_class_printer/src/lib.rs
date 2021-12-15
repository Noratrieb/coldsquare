use crate::ui::display_class;
use cs_parser::ClassFile;

mod ui;

/// Pretty-prints a class file
pub fn print(class_file: &ClassFile) {
    let stdout = std::io::stdout();

    if let Err(why) = display_class(stdout.lock(), class_file) {
        eprintln!("{}", why);
    }
}
