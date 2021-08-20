mod model;
#[cfg(test)]
mod test;

pub use model::*;

#[derive(Debug)]
pub struct ParseErr(String);

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

fn parse_vec<T: Parse, S: Into<usize>>(data: &mut Data, len: S) -> Result<Vec<T>> {
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

impl Parse for ClassFile {
    fn parse(data: &mut Data) -> Result<Self> {
        let magic = data.u4()?;
        assert_eq!(magic, 0xCAFEBABE);
        let minor_version = data.u2()?;
        let major_version = data.u2()?;
        let constant_pool_count = data.u2()?;
        let constant_pool = parse_vec(data, constant_pool_count - 1)?; // the minus one is important
        let access_flags = data.u2()?;
        let this_class = data.u2()?;
        let super_class = data.u2()?;
        let interfaces_count = data.u2()?;
        let interfaces = parse_vec(data, interfaces_count)?;
        let fields_count = data.u2()?;
        let fields = parse_vec(data, fields_count)?;
        let method_count = data.u2()?;
        let methods = parse_vec(data, method_count)?;
        let attributes_count = data.u2()?;
        let attributes = parse_vec(data, attributes_count)?;

        let mut class = Self {
            magic,
            minor_version,
            major_version,
            constant_pool_count,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces_count,
            interfaces,
            fields_count,
            fields,
            method_count,
            methods,
            attributes_count,
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
            7 => Self::Class {
                tag,
                name_index: data.u2()?,
            },
            9 => Self::Fieldref {
                tag,
                class_index: data.u2()?,
                name_and_type_index: data.u2()?,
            },
            10 => Self::MethodRef {
                tag,
                class_index: data.u2()?,
                name_and_type_index: data.u2()?,
            },
            11 => Self::InterfaceMethodref {
                tag,
                class_index: data.u2()?,
                name_and_type_index: data.u2()?,
            },
            8 => Self::String {
                tag,
                string_index: data.u2()?,
            },
            3 => Self::Integer {
                tag,
                bytes: data.u4()?,
            },
            4 => Self::Float {
                tag,
                bytes: data.u4()?,
            },
            5 => Self::Long {
                tag,
                high_bytes: data.u4()?,
                low_bytes: data.u4()?,
            },
            6 => Self::Double {
                tag,
                high_bytes: data.u4()?,
                low_bytes: data.u4()?,
            },
            12 => Self::NameAndType {
                tag,
                name_index: data.u2()?,
                descriptor_index: data.u2()?,
            },
            1 => Self::Utf8 {
                tag,
                length: data.u2()?,
                bytes: String::from_utf8(parse_vec(data, data.last_u2()?)?)
                    .map_err(|err| ParseErr(format!("Invalid utf8 in CpInfo::Utf8: {}", err)))?,
            },
            15 => Self::MethodHandle {
                tag,
                reference_kind: data.u1()?,
                reference_index: data.u2()?,
            },
            16 => Self::MethodType {
                tag,
                descriptor_index: data.u2()?,
            },
            18 => Self::InvokeDynamic {
                tag,
                bootstrap_method_attr_index: data.u2()?,
                name_and_type_index: data.u2()?,
            },
            _ => Err(ParseErr(format!("Invalid CPInfo tag: {}", tag)))?,
        })
    }
}

impl Parse for FieldInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            access_flags: data.u2()?,
            name_index: data.u2()?,
            descriptor_index: data.u2()?,
            attributes_count: data.u2()?,
            attributes: parse_vec(data, data.last_u2()?)?,
        })
    }
}

impl Parse for MethodInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            access_flags: data.u2()?,
            name_index: data.u2()?,
            descriptor_index: data.u2()?,
            attributes_count: data.u2()?,
            attributes: parse_vec(data, data.last_u2()?)?,
        })
    }
}

impl Parse for AttributeInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            attribute_name_index: data.u2()?,
            attribute_length: data.u4()?,
            inner: AttributeInfoInner::Unknown {
                attribute_content: parse_vec(data, data.last_u4()? as usize)?,
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
                locals: parse_vec(data, data.last_u2()?)?,
            },
            255 => Self::FullFrame {
                frame_type,
                offset_delta: data.u2()?,
                number_of_locals: data.u2()?,
                locals: parse_vec(data, data.last_u2()?)?,
                number_of_stack_items: data.u2()?,
                stack: parse_vec(data, data.last_u2()?)?,
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
                cpool_index: data.u2()?,
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
            inner_class_info_index: data.u2()?,
            outer_class_info_index: data.u2()?,
            inner_class_name_index: data.u2()?,
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
            name_index: data.u2()?,
            descriptor_or_signature_index: data.u2()?,
            index: data.u2()?,
        })
    }
}

impl Parse for Annotation {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            type_index: data.u2()?,
            num_element_value_pairs: data.u2()?,
            element_value_pairs: parse_vec(data, data.last_u2()?)?,
        })
    }
}

impl Parse for AnnotationElementValuePair {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            element_name_index: data.u2()?,
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
                Self::ConstValueIndex { index: data.u2()? }
            }
            'e' => Self::EnumConstValue {
                type_name_index: data.u2()?,
                const_name_index: data.u2()?,
            },
            'c' => Self::ClassInfoIndex { index: data.u2()? },
            '@' => Self::AnnotationValue {
                annotation: Box::new(Annotation::parse(data)?),
            },
            '[' => Self::ArrayValue {
                num_values: data.u2()?,
                values: parse_vec(data, data.last_u2()?)?,
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
            num_annotations: data.u2()?,
            annotations: parse_vec(data, data.last_u2()?)?,
        })
    }
}

impl Parse for BootstrapMethod {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            bootstrap_method_ref: data.u2()?,
            num_bootstrap_arguments: data.u2()?,
            bootstrap_arguments: parse_vec(data, data.last_u2()?)?,
        })
    }
}

fn resolve_attributes(class: &mut ClassFile) -> Result<()> {
    let pool = &class.constant_pool;

    class
        .attributes
        .iter_mut()
        .map(|attr| AttributeInfo::resolve_attribute(attr, pool))
        .collect::<Result<Vec<()>>>()?;

    Ok(())
}

impl AttributeInfo {
    fn resolve_attribute(&mut self, pool: &[CpInfo]) -> Result<()> {
        // this is a borrow checker hack, but it works :(
        let attr = std::mem::replace(
            self,
            AttributeInfo {
                attribute_name_index: 0,
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
        let info = match pool.get(index as usize - 1) {
            Some(CpInfo::Utf8 { bytes, .. }) => bytes,
            Some(_) => return Err(ParseErr("Attribute name is not CpInfo::Utf8".to_string())),
            _ => return Err(ParseErr("Constant Pool index out of Bounds".to_string())),
        };

        let mut data = Data::new(&content);
        self.resolve_attribute_inner(index, len, info, &mut data)
    }

    fn resolve_attribute_inner(
        &mut self,
        attribute_name_index: u16,
        attribute_length: u32,
        name: &str,
        data: &mut Data,
    ) -> Result<()> {
        let _ = std::mem::replace(
            self,
            match name {
                "ConstantValue" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::ConstantValue {
                        constantvalue_index: data.u2()?,
                    },
                },
                "Code" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::Code {
                        max_stack: data.u2()?,
                        max_locals: data.u2()?,
                        code_length: data.u4()?,
                        code: parse_vec(data, data.last_u4()? as usize)?,
                        exception_table_length: data.u2()?,
                        exception_table: parse_vec(data, data.last_u2()?)?,
                        attributes_count: data.u2()?,
                        attributes: parse_vec(data, data.last_u2()?)?,
                    },
                },
                "StackMapTable" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::StackMapTable {
                        number_of_entries: data.u2()?,
                        entries: parse_vec(data, data.last_u2()?)?,
                    },
                },
                "Exceptions" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::Exceptions {
                        number_of_exceptions: data.u2()?,
                        exception_index_table: parse_vec(data, data.last_u2()?)?,
                    },
                },
                "InnerClasses" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::InnerClasses {
                        number_of_classes: data.u2()?,
                        classes: parse_vec(data, data.last_u2()?)?,
                    },
                },
                "EnclosingMethod" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::EnclosingMethod {
                        class_index: data.u2()?,
                        method_index: data.u2()?,
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
                        signature_index: data.u2()?,
                    },
                },
                "SourceFile" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::SourceFile {
                        sourcefile_index: data.u2()?,
                    },
                },
                "SourceDebugExtension" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::SourceDebugExtension {
                        debug_extension: parse_vec(data, data.last_u2()?)?,
                    },
                },
                "LineNumberTable" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::LineNumberTable {
                        line_number_table_length: data.u2()?,
                        line_number_table: parse_vec(data, data.last_u2()?)?,
                    },
                },
                "LocalVariableTable" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::LocalVariableTable {
                        local_variable_table_length: data.u2()?,
                        local_variable_table: parse_vec(data, data.last_u2()?)?,
                    },
                },
                "LocalVariableTypeTable" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::LocalVariableTypeTable {
                        local_variable_table_length: data.u2()?,
                        local_variable_table: parse_vec(data, data.last_u2()?)?,
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
                        num_annotations: data.u2()?,
                        annotations: parse_vec(data, data.last_u2()?)?,
                    },
                },
                "RuntimeInvisibleAnnotations" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::RuntimeInvisibleAnnotations {
                        num_annotations: data.u2()?,
                        annotations: parse_vec(data, data.last_u2()?)?,
                    },
                },
                "RuntimeVisibleParameterAnnotations" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::RuntimeVisibleParameterAnnotations {
                        num_parameters: data.u1()?,
                        parameter_annotations: parse_vec(data, data.last_u1()?)?,
                    },
                },
                "RuntimeInvisibleParameterAnnotations" => Self {
                    attribute_name_index,
                    attribute_length,
                    inner: AttributeInfoInner::RuntimeInvisibleParameterAnnotations {
                        num_parameters: data.u1()?,
                        parameter_annotations: parse_vec(data, data.last_u1()?)?,
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
                        num_bootstrap_methods: data.u2()?,
                        bootstrap_methods: parse_vec(data, data.last_u2()?)?,
                    },
                },
                name => return Err(ParseErr(format!("Invalid Attribute name: {}", name))),
            },
        );

        Ok(())
    }
}
