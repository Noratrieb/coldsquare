use super::*;

#[test]
fn data_u1() {
    let bytes = [0xff, 0x00];
    let mut data = Data {
        data: &bytes,
        pointer: 0,
    };
    assert_eq!(data.u1().unwrap(), 0xff);
    assert_eq!(data.u1().unwrap(), 0x00);
    assert_eq!(data.last_u1().unwrap(), 0x00);
}

#[test]
fn data_u2() {
    let bytes = [0xff, 0x33, 0x11, 0x00];
    let mut data = Data {
        data: &bytes,
        pointer: 0,
    };
    assert_eq!(data.u2().unwrap(), 0xff33);
    assert_eq!(data.u2().unwrap(), 0x1100);
    assert_eq!(data.last_u2().unwrap(), 0x1100);
}

#[test]
fn data_u4() {
    let bytes = [0xff, 0x33, 0x11, 0x00];
    let mut data = Data {
        data: &bytes,
        pointer: 0,
    };
    assert_eq!(data.u4().unwrap(), 0xff331100);
    assert_eq!(data.last_u4().unwrap(), 0xff331100);
}

#[test]
fn parse_empty_class() {
    let class = include_bytes!("../../testdata/Test.class");
    let parsed = parse_class_file(class).unwrap();

    assert_eq!(parsed.minor_version, 0);
    assert_eq!(parsed.major_version, 0x003b);
    assert_eq!(parsed.constant_pool.len() + 1, 0x000d);
    assert_eq!(parsed.constant_pool.len(), 12);
    assert_eq!(
        parsed.constant_pool,
        vec![
            CpInfo {
                tag: 0x0a,
                inner: CpInfoInner::MethodRef(cp_info::MethodRef {
                    class_index: 2.into(),
                    name_and_type_index: 3.into(),
                })
            },
            CpInfo {
                tag: 7,
                inner: CpInfoInner::Class(cp_info::Class {
                    name_index: 4.into(),
                })
            },
            CpInfo {
                tag: 0xc,
                inner: CpInfoInner::NameAndType(cp_info::NameAndType {
                    name_index: 5.into(),
                    descriptor_index: 6.into(),
                })
            },
            CpInfo {
                tag: 1,
                inner: CpInfoInner::Utf8(cp_info::Utf8 {
                    length: 0x10,
                    bytes: "java/lang/Object".to_string()
                })
            },
            CpInfo {
                tag: 1,
                inner: CpInfoInner::Utf8(cp_info::Utf8 {
                    length: 6,
                    bytes: "<init>".to_string()
                })
            },
            CpInfo {
                tag: 1,
                inner: CpInfoInner::Utf8(cp_info::Utf8 {
                    length: 3,
                    bytes: "()V".to_string()
                })
            },
            CpInfo {
                tag: 7,
                inner: CpInfoInner::Class(cp_info::Class {
                    name_index: 8.into(),
                })
            },
            CpInfo {
                tag: 1,
                inner: CpInfoInner::Utf8(cp_info::Utf8 {
                    length: 4,
                    bytes: "Test".to_string()
                })
            },
            CpInfo {
                tag: 1,
                inner: CpInfoInner::Utf8(cp_info::Utf8 {
                    length: 4,
                    bytes: "Code".to_string()
                })
            },
            CpInfo {
                tag: 1,
                inner: CpInfoInner::Utf8(cp_info::Utf8 {
                    length: 15,
                    bytes: "LineNumberTable".to_string()
                })
            },
            CpInfo {
                tag: 1,
                inner: CpInfoInner::Utf8(cp_info::Utf8 {
                    length: 10,
                    bytes: "SourceFile".to_string()
                })
            },
            CpInfo {
                tag: 1,
                inner: CpInfoInner::Utf8(cp_info::Utf8 {
                    length: 9,
                    bytes: "Test.java".to_string()
                })
            }
        ]
    );
    assert_eq!(parsed.access_flags, 0x0021);
    assert_eq!(parsed.this_class, 7.into());
    assert_eq!(parsed.super_class, 2.into());
    assert_eq!(parsed.interfaces.len(), 0);
    assert_eq!(parsed.interfaces, vec![]);
    assert_eq!(parsed.fields.len(), 0);
    assert_eq!(parsed.fields, vec![]);
    assert_eq!(parsed.methods.len(), 1);
    assert_eq!(parsed.methods[0].access_flags, 1);
    assert_eq!(parsed.methods[0].name_index, 5.into());
    assert_eq!(parsed.methods[0].descriptor_index, 6.into());
    assert_eq!(parsed.methods[0].attributes.len(), 1);
}

#[test]
fn more_complex_file() {
    let class = include_bytes!("../../testdata/Test2.class");
    let parsed = parse_class_file(class).unwrap();
    assert_eq!(parsed.magic, 0xCAFEBABE);
}
