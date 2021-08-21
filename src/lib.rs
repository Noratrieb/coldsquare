use crate::parse::{ClassFile, ParseErr};
use std::error::Error;

mod execute;
mod parse;
mod ui;

pub fn parse_class_file(file: &[u8]) -> Result<ClassFile, ParseErr> {
    parse::parse_class_file(file)
}

pub fn display_class<W>(w: W, class: &ClassFile) -> Result<(), Box<dyn Error>>
where
    W: std::io::Write,
{
    ui::display_class(w, class)
}
