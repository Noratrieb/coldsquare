use crate::parse::{ClassFile, ParseErr};

mod execute;
mod parse;

pub fn parse_class_file(file: Vec<u8>) -> Result<ClassFile, ParseErr> {
    parse::parse_class_file(file)
}
