use crate::parse::{ClassFile, ParseErr};

mod execute;
mod parse;
mod ui;

pub fn parse_class_file(file: &[u8]) -> Result<ClassFile, ParseErr> {
    parse::parse_class_file(file)
}

pub fn display_class<W>(w: W, class: &ClassFile) -> std::io::Result<()>
where
    W: std::io::Write,
{
    ui::display_class(w, class)
}
