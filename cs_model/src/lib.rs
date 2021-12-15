#![allow(dead_code)]

#[cfg(test)]
mod test;

use std::borrow::Cow;
use std::str::FromStr;

#[derive(Debug)]
pub struct ParseErr(pub Cow<'static, str>);

impl ParseErr {
    pub fn str(str: &'static str) -> Self {
        Self(Cow::Borrowed(str))
    }
    pub fn string(str: String) -> Self {
        Self(Cow::Owned(str))
    }
}

/// A field descriptor for the type of a field in a class
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FieldDescriptor(pub FieldType);

/// The type of a field or method parameter
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum FieldType {
    /// B
    Byte,
    /// C
    Char,
    /// D
    Double,
    /// F
    Float,
    /// I
    Int,
    /// J
    Long,
    /// L `ClassName` ;
    Object(String),
    /// S
    Short,
    /// Z
    Boolean,
    /// [
    Array(Box<Self>),
}

/// A method descriptor for the type of a method in a class
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MethodDescriptor {
    parameters: Vec<FieldType>,
    return_: MethodType,
}

/// The type of a method
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum MethodType {
    Some(FieldType),
    /// V
    Void,
}

impl FromStr for FieldDescriptor {
    type Err = ParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(FieldType::from_char_iter(&mut s.chars())?))
    }
}

impl FieldType {
    /// Consumes as much chars as needed from the char iterator and tries to parse itself
    pub fn from_char_iter<I>(chars: &mut I) -> Result<Self, ParseErr>
    where
        I: Iterator<Item = char>,
    {
        let first = chars.next().ok_or_else(|| ParseErr::str("Empty string"))?;
        Ok(match first {
            'B' => Self::Byte,
            'C' => Self::Char,
            'D' => Self::Double,
            'F' => Self::Float,
            'I' => Self::Int,
            'J' => Self::Long,
            'L' => Self::Object({
                let mut name = String::with_capacity(32); // we can expect ClassNames to be at least this long
                loop {
                    let char = chars
                        .next()
                        .ok_or_else(|| ParseErr::str("Expected ; before end of string"))?;

                    if char == ';' {
                        break;
                    };
                    name.push(char);
                }
                name
            }),
            'S' => Self::Short,
            'Z' => Self::Boolean,
            '[' => Self::Array(Box::new(Self::from_char_iter(chars)?)),
            c => {
                return Err(ParseErr::string(format!(
                    "Invalid char in field descriptor {}",
                    c
                )))
            }
        })
    }
}

impl FromStr for MethodDescriptor {
    type Err = ParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars().peekable();
        if chars.next().ok_or_else(|| ParseErr::str("Empty string"))? != '(' {
            return Err(ParseErr::str("Needs to start with '('"));
        }

        let mut parameters = Vec::new();

        loop {
            if let Some(')') = chars.peek() {
                let _ = chars.next(); // consume the )
                break;
            }
            parameters.push(FieldType::from_char_iter(&mut chars)?);
        }

        let return_ = if let Some('V') = chars.peek() {
            MethodType::Void
        } else {
            MethodType::Some(FieldType::from_char_iter(&mut chars)?)
        };

        Ok(Self {
            parameters,
            return_,
        })
    }
}
