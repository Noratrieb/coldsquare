use super::*;

///
/// An index into the constant pool of the class
/// `T` -> What type the target value is supposed to be. Create an enum if multiple values can be there
#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FromPool<T> {
    inner: u2,
    _marker: PhantomData<fn() -> T>,
}

// could probably be derived if I chose a better marker
impl<T: Clone> Copy for FromPool<T> {}

impl<T> std::ops::Deref for FromPool<T> {
    type Target = u2;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> From<u2> for FromPool<T> {
    fn from(n: u2) -> Self {
        Self {
            inner: n,
            _marker: PhantomData,
        }
    }
}

pub trait FromCpInfo<'a> {
    type Target;
    fn from_cp_info(info: &'a CpInfo) -> Result<Self::Target, ParseErr>;
}

impl<'a, T> FromCpInfo<'a> for Option<T>
where
    T: FromCpInfo<'a, Target = &'a T>,
    T: 'a,
{
    type Target = Option<&'a T>;

    fn from_cp_info(info: &'a CpInfo) -> Result<Self::Target, ParseErr> {
        Ok(T::from_cp_info(info).ok())
    }
}

impl<'a, T> FromPool<T>
where
    T: FromCpInfo<'a>,
{
    pub fn get(&self, pool: &'a [CpInfo]) -> Result<T::Target, ParseErr> {
        T::from_cp_info(&pool[self.inner as usize - 1])
    }
}

macro_rules! impl_try_from_cp {
    ($($name:ident),*) => {
        $(
            impl<'a> FromCpInfo<'a> for $name {
                type Target = &'a Self;

                fn from_cp_info(info: &'a CpInfo) -> Result<Self::Target, ParseErr> {
                    match &info.inner {
                        CpInfoInner::$name(class) => Ok(class),
                        kind => Err(ParseErr(format!(
                            concat!("Expected '", stringify!($name), "', found '{:?}'"),
                            kind
                        ))),
                    }
                }
            }
        )*
    };
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Class {
    /// Entry must be `Utf8`
    pub name_index: FromPool<cp_info::Utf8>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Fieldref {
    /// May be a class or interface type
    pub class_index: FromPool<cp_info::Class>,
    /// Entry must be `NameAndType`
    pub name_and_type_index: FromPool<cp_info::NameAndType>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MethodRef {
    /// Must be a class type
    pub class_index: FromPool<cp_info::Class>,
    /// Entry must be `NameAndType`
    pub name_and_type_index: FromPool<cp_info::NameAndType>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct InterfaceMethodref {
    /// Must be an interface type
    pub class_index: FromPool<cp_info::Class>,
    /// Entry must be `NameAndType`
    pub name_and_type_index: FromPool<cp_info::NameAndType>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct String {
    /// Entry must be `Utf8`
    pub string_index: FromPool<cp_info::Utf8>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Integer {
    // Big endian
    pub bytes: u4,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Float {
    /// IEEE 754 floating-point single format, big endian
    pub bytes: u4,
}

/// 8 byte constants take up two spaces in the constant pool
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Long {
    /// Big endian
    pub high_bytes: u4,
    /// Big endian
    pub low_bytes: u4,
}

/// 8 byte constants take up two spaces in the constant pool
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Double {
    /// IEEE 754 floating-point double format, big endian
    pub high_bytes: u4,
    /// IEEE 754 floating-point double format, big endian
    pub low_bytes: u4,
}

/// Any field or method, without the class it belongs to
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct NameAndType {
    /// Entry must be `Utf8`
    pub name_index: FromPool<cp_info::Utf8>,
    /// Entry must be `Utf8`
    pub descriptor_index: FromPool<cp_info::Utf8>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Utf8 {
    /// Contains modified UTF-8
    pub bytes: std::string::String,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MethodHandle {
    /// The kind of method handle (0-9)
    /// If the kind is 1-4, the entry must be `FieldRef`. If the kind is 5-8, the entry must be `MethodRef`
    /// If the kind is 9, the entry must be `InterfaceMethodRef`
    pub reference_kind: u1,
    pub reference_index: MethodHandleIndex,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MethodHandleIndex {
    Field(FromPool<cp_info::Fieldref>),
    Method(FromPool<cp_info::MethodInfo>),
    Interface(FromPool<cp_info::InterfaceMethodref>),
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MethodType {
    /// Entry must be `Utf8`
    pub descriptor_index: FromPool<cp_info::Utf8>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct InvokeDynamic {
    /// Must be a valid index into the `bootstrap_methods` array of the bootstrap method table of this class field
    pub bootstrap_method_attr_index: u2,
    /// Entry must `NameAndType`
    pub name_and_type_index: FromPool<cp_info::NameAndType>,
}

// default implementations

impl_try_from_cp!(
    Class,
    Fieldref,
    MethodRef,
    InterfaceMethodref,
    String,
    Integer,
    Float,
    Long,
    Double,
    NameAndType,
    MethodHandle,
    MethodType,
    InvokeDynamic
);

// custom implementations
impl<'a> FromCpInfo<'a> for Utf8 {
    type Target = &'a str;

    fn from_cp_info(info: &'a CpInfo) -> Result<Self::Target, ParseErr> {
        match &info.inner {
            CpInfoInner::Utf8(class) => Ok(&class.bytes),
            kind => Err(ParseErr(format!(
                concat!("Expected '", stringify!($name), "', found '{:?}'"),
                kind
            ))),
        }
    }
}
