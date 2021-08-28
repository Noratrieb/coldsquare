use super::*;

#[test]
fn field_descriptor() {
    let descriptors = [
        FieldDescriptor::from_str("B").unwrap(),
        FieldDescriptor::from_str("C").unwrap(),
        FieldDescriptor::from_str("D").unwrap(),
        FieldDescriptor::from_str("F").unwrap(),
        FieldDescriptor::from_str("I").unwrap(),
        FieldDescriptor::from_str("J").unwrap(),
        FieldDescriptor::from_str("S").unwrap(),
        FieldDescriptor::from_str("Z").unwrap(),
        FieldDescriptor::from_str("[B").unwrap(),
        FieldDescriptor::from_str("[[Z").unwrap(),
        FieldDescriptor::from_str("Ljava/lang/String;").unwrap(),
        FieldDescriptor::from_str("[[[Ljava/lang/String;").unwrap(),
    ];

    type FT = FieldType;

    let expected_descriptors = [
        FieldDescriptor(FT::Byte),
        FieldDescriptor(FT::Char),
        FieldDescriptor(FT::Double),
        FieldDescriptor(FT::Float),
        FieldDescriptor(FT::Int),
        FieldDescriptor(FT::Long),
        FieldDescriptor(FT::Short),
        FieldDescriptor(FT::Boolean),
        FieldDescriptor(FT::Array(Box::new(FT::Byte))),
        FieldDescriptor(FT::Array(Box::new(FT::Array(Box::new(FT::Boolean))))),
        FieldDescriptor(FT::Object("java/lang/String".to_string())),
        FieldDescriptor(FT::Array(Box::new(FT::Array(Box::new(FT::Array(
            Box::new(FT::Object("java/lang/String".to_string())),
        )))))),
    ];

    let invalid_descriptors = ["", "Q", "[]", "[", "Ljava/lang/String", "L", "[[[Ljava"];

    descriptors
        .iter()
        .zip(expected_descriptors.iter())
        .for_each(|(a, b)| assert_eq!(a, b));

    invalid_descriptors
        .iter()
        .map(|d| FieldDescriptor::from_str(d))
        .for_each(|rs| {
            if rs.is_ok() {
                panic!("Successfully parsed invalid result, {:?}", rs);
            }
        });
}

#[test]
fn method_descriptor() {
    let descriptors = vec![
        MethodDescriptor::from_str("()V").unwrap(),
        MethodDescriptor::from_str("(B)V").unwrap(),
        MethodDescriptor::from_str("([ZZ)Ljava/lang/Object;").unwrap(),
        MethodDescriptor::from_str("(IDLjava/lang/Thread;)Ljava/lang/Object;").unwrap(),
        MethodDescriptor::from_str("(BBBBBBBBBB)B").unwrap(),
        MethodDescriptor::from_str("()Z").unwrap(),
    ];

    type FT = FieldType;

    let expected_descriptors = [
        MethodDescriptor {
            parameters: vec![],
            return_: MethodType::Void,
        },
        MethodDescriptor {
            parameters: vec![FT::Byte],
            return_: MethodType::Void,
        },
        MethodDescriptor {
            parameters: vec![FT::Array(Box::new(FT::Boolean)), FT::Boolean],
            return_: MethodType::Some(FT::Object("java/lang/Object".to_string())),
        },
        MethodDescriptor {
            parameters: vec![
                FT::Int,
                FT::Double,
                FT::Object("java/lang/Thread".to_string()),
            ],
            return_: MethodType::Some(FT::Object("java/lang/Object".to_string())),
        },
        MethodDescriptor {
            parameters: vec![
                FT::Byte,
                FT::Byte,
                FT::Byte,
                FT::Byte,
                FT::Byte,
                FT::Byte,
                FT::Byte,
                FT::Byte,
                FT::Byte,
                FT::Byte,
            ],
            return_: MethodType::Some(FT::Byte),
        },
        MethodDescriptor {
            parameters: vec![],
            return_: MethodType::Some(FT::Boolean),
        },
    ];

    let invalid_descriptors = ["()", "(V)V", ")V", "(;)Z", "(java/lang/StringZ)", "V"];

    invalid_descriptors
        .iter()
        .map(|d| MethodDescriptor::from_str(d))
        .for_each(|rs| {
            if rs.is_ok() {
                panic!("Successfully parsed invalid result, {:?}", rs);
            }
        });

    descriptors
        .iter()
        .zip(expected_descriptors.iter())
        .for_each(|(a, b)| assert_eq!(a, b));
}
