mod model;
#[cfg(test)]
mod test;

pub use model::*;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ParseErr(String);

impl Display for ParseErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not parse class file: {}", self.0)
    }
}

impl std::error::Error for ParseErr {}

pub type Result<T> = std::result::Result<T, ParseErr>;

#[derive(Clone)]
struct Data<'a> {
    data: &'a [u1],
    pointer: usize,
}

pub fn parse_class_file(data: &[u1]) -> Result<ClassFile> {
    let mut data = Data::new(data);
    ClassFile::parse(&mut data)
}

impl<'a> Data<'a> {
    fn new(data: &'a [u1]) -> Self {
        Data { data, pointer: 0 }
    }

    fn u1(&mut self) -> Result<u1> {
        let item = self.data.get(self.pointer).cloned();
        self.pointer += 1;
        item.ok_or(ParseErr("No u1 left".to_string()))
    }

    fn u2(&mut self) -> Result<u2> {
        Ok(((self.u1()? as u2) << 8) | self.u1()? as u2)
    }

    fn cp<T>(&mut self) -> Result<FromPool<T>> {
        Ok(self.u2()?.into())
    }

    fn u4(&mut self) -> Result<u4> {
        Ok(((self.u2()? as u4) << 16) | self.u2()? as u4)
    }

    fn last_u1(&self) -> Result<u1> {
        self.data
            .get(self.pointer - 1)
            .cloned()
            .ok_or(ParseErr("Last u1 not found".to_string()))
    }

    fn last_u2(&self) -> Result<u2> {
        let last2u1 = self
            .data
            .get(self.pointer - 2)
            .cloned()
            .ok_or(ParseErr("Last u2 not found".to_string()))?;
        Ok(((last2u1 as u2) << 8) | self.last_u1()? as u2)
    }

    fn last_u4(&self) -> Result<u4> {
        let last2u1 = self
            .data
            .get(self.pointer - 3)
            .cloned()
            .ok_or(ParseErr("Last 2 u1 in last u4 not found".to_string()))?;
        let last3u1 = self
            .data
            .get(self.pointer - 4)
            .cloned()
            .ok_or(ParseErr("Last 3 u1 in last u4 not found".to_string()))?;
        Ok(((last3u1 as u4) << 24) | ((last2u1 as u4) << 16) | self.last_u2()? as u4)
    }
}

trait Parse {
    fn parse(data: &mut Data) -> Result<Self>
    where
        Self: Sized;
}

fn parse_vec<T: Parse, S: Into<usize>>(len: S, data: &mut Data) -> Result<Vec<T>> {
    let len = len.into();
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(T::parse(data)?);
    }
    Ok(vec)
}

macro_rules! parse_primitive {
    ($($value:ident),*) => {
        $(impl Parse for $value {
            fn parse(data: &mut Data) -> Result<Self>
            where
                Self: Sized,
            {
                data.$value()
            }
        })*
    };
}

parse_primitive!(u1, u2, u4);

impl<T> Parse for FromPool<T> {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(data.u2()?.into())
    }
}

impl Parse for ClassFile {
    fn parse(data: &mut Data) -> Result<Self> {
        let magic = data.u4()?;
        assert_eq!(magic, 0xCAFEBABE);
        let minor_version = data.u2()?;
        let major_version = data.u2()?;
        let constant_pool = parse_vec(data.u2()? - 1, data)?; // the minus one is important
        let access_flags = data.u2()?;
        let this_class = data.cp()?;
        let super_class = data.cp()?;
        let interfaces = parse_vec(data.u2()?, data)?;
        let fields = parse_vec(data.u2()?, data)?;
        let methods = parse_vec(data.u2()?, data)?;
        let attributes = parse_vec(data.u2()?, data)?;

        let mut class = Self {
            magic,
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        };
        resolve_attributes(&mut class)?;
        Ok(class)
    }
}

impl Parse for CpInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        let tag = data.u1()?;

        Ok(match tag {
            7 => Self {
                tag,
                inner: CpInfoInner::Class(cp_info::Class {
                    name_index: data.cp()?,
                }),
            },
            9 => Self {
                tag,
                inner: CpInfoInner::Fieldref(cp_info::Fieldref {
                    class_index: data.cp()?,
                    name_and_type_index: data.cp()?,
                }),
            },
            10 => Self {
                tag,
                inner: CpInfoInner::MethodRef(cp_info::MethodRef {
                    class_index: data.cp()?,
                    name_and_type_index: data.cp()?,
                }),
            },
            11 => Self {
                tag,
                inner: CpInfoInner::InterfaceMethodref(cp_info::InterfaceMethodref {
                    class_index: data.cp()?,
                    name_and_type_index: data.cp()?,
                }),
            },
            8 => Self {
                tag,
                inner: CpInfoInner::String(cp_info::String {
                    string_index: data.cp()?,
                }),
            },
            3 => Self {
                tag,
                inner: CpInfoInner::Integer(cp_info::Integer { bytes: data.u4()? }),
            },
            4 => Self {
                tag,
                inner: CpInfoInner::Float(cp_info::Float { bytes: data.u4()? }),
            },
            5 => Self {
                tag,
                inner: CpInfoInner::Long(cp_info::Long {
                    high_bytes: data.u4()?,
                    low_bytes: data.u4()?,
                }),
            },
            6 => Self {
                tag,
                inner: CpInfoInner::Double(cp_info::Double {
                    high_bytes: data.u4()?,
                    low_bytes: data.u4()?,
                }),
            },
            12 => Self {
                tag,
                inner: CpInfoInner::NameAndType(cp_info::NameAndType {
                    name_index: data.cp()?,
                    descriptor_index: data.cp()?,
                }),
            },
            1 => Self {
                tag,
                inner: CpInfoInner::Utf8(cp_info::Utf8 {
                    bytes: String::from_utf8(parse_vec(data.u2()?, data)?).map_err(|err| {
                        ParseErr(format!("Invalid utf8 in CpInfo::Utf8: {}", err))
                    })?,
                }),
            },
            15 => Self {
                tag,
                inner: CpInfoInner::MethodHandle(cp_info::MethodHandle {
                    reference_kind: data.u1()?,
                    reference_index: match data.last_u1()? {
                        1..=4 => cp_info::MethodHandleIndex::Field(data.cp()?),
                        5..=8 => cp_info::MethodHandleIndex::Method(data.cp()?),
                        9 => cp_info::MethodHandleIndex::Interface(data.cp()?),
                        n => {
                            return Err(ParseErr(format!(
                                "Invalid MethodHandle reference kind: {}",
                                n
                            )))
                        }
                    },
                }),
            },
            16 => Self {
                tag,
                inner: CpInfoInner::MethodType(cp_info::MethodType {
                    descriptor_index: data.cp()?,
                }),
            },
            18 => Self {
                tag,
                inner: CpInfoInner::InvokeDynamic(cp_info::InvokeDynamic {
                    bootstrap_method_attr_index: data.u2()?,
                    name_and_type_index: data.cp()?,
                }),
            },
            _ => Err(ParseErr(format!("Invalid CPInfo tag: {}", tag)))?,
        })
    }
}

impl Parse for FieldInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            access_flags: data.u2()?,
            name_index: data.cp()?,
            descriptor_index: data.cp()?,
            attributes: parse_vec(data.u2()?, data)?,
        })
    }
}

impl Parse for MethodInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            access_flags: data.u2()?,
            name_index: data.cp()?,
            descriptor_index: data.cp()?,
            attributes: parse_vec(data.u2()?, data)?,
        })
    }
}

impl Parse for AttributeInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            attribute_name_index: data.cp()?,
            attribute_length: data.u4()?,
            inner: AttributeInfoInner::Unknown {
                attribute_content: parse_vec(data.last_u4()? as usize, data)?,
            },
        })
    }
}

impl Parse for AttributeCodeException {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            start_pc: data.last_u2()?,
            end_pc: data.last_u2()?,
            handler_pc: data.last_u2()?,
            catch_type: data.last_u2()?,
        })
    }
}

impl Parse for StackMapFrame {
    fn parse(data: &mut Data) -> Result<Self> {
        let frame_type = data.u1()?;

        Ok(match frame_type {
            0..=63 => Self::SameFrame { frame_type },
            64..=127 => Self::SameLocals1StackItemFrame {
                frame_type,
                stack: VerificationTypeInfo::parse(data)?,
            },
            247 => Self::SameLocals1StackItemFrameExtended {
                frame_type,
                offset_delta: data.u2()?,
                stack: VerificationTypeInfo::parse(data)?,
            },
            246..=250 => Self::ChopFrame {
                frame_type,
                offset_delta: data.u2()?,
            },
            251 => Self::SameFrameExtended {
                frame_type,
                offset_delta: data.u2()?,
            },
            252..=254 => Self::AppendFrame {
                frame_type,
                offset_delta: data.u2()?,
                locals: parse_vec(data.last_u2()?, data)?,
            },
            255 => Self::FullFrame {
                frame_type,
                offset_delta: data.u2()?,
                locals: parse_vec(data.u2()?, data)?,
                stack: parse_vec(data.u2()?, data)?,
            },
            _ => Err(ParseErr(format!(
                "Invalid StackMapFrame type: {}",
                frame_type
            )))?,
        })
    }
}

impl Parse for VerificationTypeInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        let tag = data.u1()?;
        Ok(match tag {
            0 => Self::Top { tag },
            1 => Self::Integer { tag },
            2 => Self::Float { tag },
            4 => Self::Long { tag },
            3 => Self::Double { tag },
            5 => Self::Null { tag },
            6 => Self::UninitializedThis { tag },
            7 => Self::Object {
                tag,
                cpool_index: data.u2()?.into(),
            },
            8 => Self::Uninitialized {
                tag,
                offset: data.u2()?,
            },
            _ => Err(ParseErr(format!(
                "Invalid VerificationTypeInfo tag: {}",
                tag
            )))?,
        })
    }
}

impl Parse for AttributeInnerClass {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            inner_class_info_index: data.cp()?,
            outer_class_info_index: data.cp()?,
            inner_class_name_index: data.cp()?,
            inner_class_access_flags: data.u2()?,
        })
    }
}

impl Parse for AttributeLineNumber {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            start_pc: data.u2()?,
            line_number: data.u2()?,
        })
    }
}

impl Parse for AttributeLocalVariableTable {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            start_pc: data.u2()?,
            length: data.u2()?,
            name_index: data.cp()?,
            descriptor_or_signature_index: data.cp()?,
            index: data.u2()?,
        })
    }
}

impl Parse for Annotation {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            type_index: data.cp()?,
            num_element_value_pairs: data.u2()?,
            element_value_pairs: parse_vec(data.last_u2()?, data)?,
        })
    }
}

impl Parse for AnnotationElementValuePair {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            element_name_index: data.cp()?,
            element_name_name: AnnotationElementValue::parse(data)?,
        })
    }
}

impl Parse for AnnotationElementValue {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            tag: data.u1()?,
            value: AnnotationElementValueValue::parse(data)?,
        })
    }
}

impl Parse for AnnotationElementValueValue {
    fn parse(data: &mut Data) -> Result<Self> {
        let tag = data.last_u1()? as char;
        Ok(match tag {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => {
                Self::ConstValueIndex { index: data.cp()? }
            }
            'e' => Self::EnumConstValue {
                type_name_index: data.cp()?,
                const_name_index: data.cp()?,
            },
            'c' => Self::ClassInfoIndex { index: data.cp()? },
            '@' => Self::AnnotationValue {
                annotation: Box::new(Annotation::parse(data)?),
            },
            '[' => Self::ArrayValue {
                values: parse_vec(data.u2()?, data)?,
            },
            _ => Err(ParseErr(format!(
                "Invalid AnnotationElementValueValue tag: {}",
                tag
            )))?,
        })
    }
}

impl Parse for ParameterAnnotation {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            annotations: parse_vec(data.u2()?, data)?,
        })
    }
}

impl Parse for BootstrapMethod {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            bootstrap_method_ref: data.cp()?,
            bootstrap_arguments: parse_vec(data.u2()?, data)?,
        })
    }
}

fn resolve_attributes(class: &mut ClassFile) -> Result<()> {
    let pool = &class.constant_pool;

    class
        .attributes
        .iter_mut()
        .map(|attr| attr.resolve_attribute(pool))
        .collect::<Result<Vec<()>>>()?;

    class
        .methods
        .iter_mut()
        .map(|method| {
            method
                .attributes
                .iter_mut()
                .map(|attr| attr.resolve_attribute(pool))
                .collect::<Result<Vec<()>>>()
        })
        .collect::<Result<Vec<_>>>()?;

    class
        .fields
        .iter_mut()
        .map(|method| {
            method
                .attributes
                .iter_mut()
                .map(|attr| attr.resolve_attribute(pool))
                .collect::<Result<Vec<()>>>()
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

impl AttributeInfo {
    fn resolve_attribute(&mut self, pool: &[CpInfo]) -> Result<()> {
        // this is a borrow checker hack, but it works :(
        let attr = std::mem::replace(
            self,
            AttributeInfo {
                attribute_name_index: 0.into(),
                attribute_length: 0,
                inner: AttributeInfoInner::__Empty,
            },
        );

        let (&index, &len, content) = match &attr {
            AttributeInfo {
                attribute_name_index,
                attribute_length,
                inner: AttributeInfoInner::Unknown { attribute_content },
            } => (attribute_name_index, attribute_length, attribute_content),
            _ => unreachable!("Attribute already resolved"),
        };
        let info = match pool.get((*index) as usize - 1) {
            Some(CpInfo {
                inner: CpInfoInner::Utf8(cp_info::Utf8 { bytes, .. }),
                ..
            }) => bytes,
            Some(_) => return Err(ParseErr("Attribute name is not CpInfo::Utf8".to_string())),
            _ => return Err(ParseErr("Constant Pool index out of Bounds".to_string())),
        };

        let mut data = Data::new(&content);
        self.resolve_attribute_inner(index, len, info, &mut data, pool)
    }

    fn resolve_attribute_inner(
        &mut self,
        attribute_name_index: FromPool<cp_info::Utf8>,
        attribute_length: u32,
        name: &str,
        data: &mut Data,
        pool: &[CpInfo],
    ) -> Result<()> {
        let _ = std::mem::replace(
            self,
            match name {
                "ConstantValue" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::ConstantValue {
                        constantvalue_index: data.cp()?,
                    },
                },
                "Code" => {
                    let mut code = Self {
                        attribute_name_index,
                        attribute_length,
                        inner: AttributeInfoInner::Code {
                            max_stack: data.u2()?,
                            max_locals: data.u2()?,
                            code: parse_vec(data.u4()? as usize, data)?,
                            exception_table: parse_vec(data.u2()?, data)?,
                            attributes: parse_vec(data.u2()?, data)?,
                        },
                    };
                    if let AttributeInfoInner::Code {
                        ref mut attributes, ..
                    } = code.inner
                    {
                        attributes
                            .iter_mut()
                            .map(|attr| attr.resolve_attribute(pool))
                            .collect::<Result<Vec<()>>>()?;
                    } else {
                        unreachable!()
                    }
                    code
                }
                "StackMapTable" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::StackMapTable {
                        number_of_entries: data.u2()?,
                        entries: parse_vec(data.last_u2()?, data)?,
                    },
                },
                "Exceptions" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::Exceptions {
                        exception_index_table: parse_vec(data.u2()?, data)?,
                    },
                },
                "InnerClasses" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::InnerClasses {
                        classes: parse_vec(data.u2()?, data)?,
                    },
                },
                "EnclosingMethod" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::EnclosingMethod {
                        class_index: data.cp()?,
                        method_index: data.cp()?,
                    },
                },
                "Synthetic" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::Synthetic,
                },
                "Signature" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::Signature {
                        signature_index: data.cp()?,
                    },
                },
                "SourceFile" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::SourceFile {
                        sourcefile_index: data.cp()?,
                    },
                },
                "SourceDebugExtension" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::SourceDebugExtension {
                        debug_extension: parse_vec(data.last_u2()?, data)?,
                    },
                },
                "LineNumberTable" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::LineNumberTable {
                        line_number_table: parse_vec(data.u2()?, data)?,
                    },
                },
                "LocalVariableTable" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::LocalVariableTable {
                        local_variable_table: parse_vec(data.u2()?, data)?,
                    },
                },
                "LocalVariableTypeTable" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::LocalVariableTypeTable {
                        local_variable_table: parse_vec(data.u2()?, data)?,
                    },
                },
                "Deprecated" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::Deprecated,
                },
                "RuntimeVisibleAnnotations" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::RuntimeVisibleAnnotations {
                        annotations: parse_vec(data.u2()?, data)?,
                    },
                },
                "RuntimeInvisibleAnnotations" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::RuntimeInvisibleAnnotations {
                        annotations: parse_vec(data.u2()?, data)?,
                    },
                },
                "RuntimeVisibleParameterAnnotations" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::RuntimeVisibleParameterAnnotations {
                        parameter_annotations: parse_vec(data.u1()?, data)?,
                    },
                },
                "RuntimeInvisibleParameterAnnotations" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::RuntimeInvisibleParameterAnnotations {
                        parameter_annotations: parse_vec(data.u1()?, data)?,
                    },
                },
                "AnnotationDefault" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::AnnotationDefault {
                        default_value: AnnotationElementValue {
                            tag: data.u1()?,
                            value: AnnotationElementValueValue::parse(data)?,
                        },
                    },
                },
                "BootstrapMethods" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::BootstrapMethods {
                        bootstrap_methods: parse_vec(data.u2()?, data)?,
                    },
                },
                name => return Err(ParseErr(format!("Invalid Attribute name: {}", name))),
            },
        );

        Ok(())
    }
}
