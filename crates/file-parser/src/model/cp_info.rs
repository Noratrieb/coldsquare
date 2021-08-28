use crate::{u1, u2, u4, CpInfo, CpInfoInner, ParseErr};
use std::marker::PhantomData;

///
/// An index into the constant pool of the class
/// `T` -> What type the target value is supposed to be. Create an enum if multiple values can be there
///
/// The value this is pointing at must *always* be a entry of the correct type T
/// Type checking is done at parse time, so that the value can be get with minimal overhead
#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FromPool<T> {
    inner: u2,
    _marker: PhantomData<fn() -> T>,
}

// could probably be derived if I chose a better marker
impl<T: Clone> Copy for FromPool<T> {}

impl<T> From<u2> for FromPool<T> {
    #[inline]
    fn from(n: u2) -> Self {
        Self {
            inner: n,
            _marker: PhantomData,
        }
    }
}

impl<T> FromPool<T> {
    #[inline]
    pub const fn inner(&self) -> u2 {
        self.inner
    }
}

impl<'pool, T> FromPool<T>
where
    T: FromCpInfo<'pool>,
{
    #[inline]
    pub fn get(&self, pool: &'pool [CpInfo]) -> T::Target {
        T::from_cp_info_with_index(pool, self.inner)
    }
}

impl<'pool, T> FromPool<Option<T>>
where
    T: FromCpInfo<'pool>,
{
    #[inline]
    pub fn maybe_get(&self, pool: &'pool [CpInfo]) -> Option<T::Target> {
        if self.inner == 0 {
            None
        } else {
            Some(T::from_cp_info_with_index(pool, self.inner))
        }
    }
}

pub trait ValidateCpInfo {
    /// check that the constant pool entry has the correct type
    /// `index` is the original, non-null index (it can be 0 optional constants)
    fn validate_cp_info(info: &[CpInfo], index: u2) -> Result<(), ParseErr>;
}

pub trait FromCpInfo<'pool>: ValidateCpInfo {
    type Target;
    fn from_cp_info(info: &'pool CpInfo) -> Self::Target;
    fn from_cp_info_with_index(info: &'pool [CpInfo], index: u2) -> Self::Target {
        Self::from_cp_info(&info[index as usize - 1])
    }
}

impl<'pool, T> FromCpInfo<'pool> for Option<T>
where
    T: FromCpInfo<'pool>,
{
    type Target = Option<T::Target>;

    #[inline]
    fn from_cp_info(_info: &'pool CpInfo) -> Self::Target {
        unreachable!("FromPool<Option<T>> should always be get through `from_cp_info_with_index`")
    }

    fn from_cp_info_with_index(info: &'pool [CpInfo], index: u2) -> Self::Target {
        if index == 0 {
            None
        } else {
            Some(T::from_cp_info_with_index(info, index))
        }
    }
}

impl<T> ValidateCpInfo for Option<T>
where
    T: ValidateCpInfo,
{
    fn validate_cp_info(info: &[CpInfo], index: u2) -> Result<(), ParseErr> {
        if index == 0 {
            Ok(())
        } else {
            T::validate_cp_info(info, index)
        }
    }
}

macro_rules! impl_try_from_cp {
    ($($name:ident),*) => {
        $(
            impl<'pool> FromCpInfo<'pool> for $name {
                type Target = &'pool Self;

                #[inline]
                fn from_cp_info(info: &'pool CpInfo) -> Self::Target {
                    match &info.inner {
                        CpInfoInner::$name(class) => class,
                        _kind => unreachable!(),
                    }
                }
            }

            impl ValidateCpInfo for $name {
                fn validate_cp_info(info: &[CpInfo], index: u2) -> Result<(), ParseErr> {
                    if index == 0 {
                        return Err(ParseErr("Index must not be 0".to_string()));
                    }
                    // todo this here might actually be an empty constant pool depending on whether is is still parsing the constant pool
                    // it needs to be checked after testing
                    // not now
                    // pls
                    // i hate this
                    match &info[index as usize - 1].inner {
                        CpInfoInner::$name(_) => Ok(()),
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

impl<'pool> FromCpInfo<'pool> for CpInfoInner {
    type Target = &'pool Self;

    fn from_cp_info(info: &'pool CpInfo) -> Self::Target {
        &info.inner
    }
}

impl ValidateCpInfo for CpInfoInner {
    fn validate_cp_info(_info: &[CpInfo], _index: u2) -> Result<(), ParseErr> {
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Class {
    /// Entry must be `Utf8`
    pub name_index: FromPool<Utf8>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Fieldref {
    /// May be a class or interface type
    pub class_index: FromPool<Class>,
    /// Entry must be `NameAndType`
    pub name_and_type_index: FromPool<NameAndType>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MethodRef {
    /// Must be a class type
    pub class_index: FromPool<Class>,
    /// Entry must be `NameAndType`
    pub name_and_type_index: FromPool<NameAndType>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct InterfaceMethodref {
    /// Must be an interface type
    pub class_index: FromPool<Class>,
    /// Entry must be `NameAndType`
    pub name_and_type_index: FromPool<NameAndType>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct String {
    /// Entry must be `Utf8`
    pub string_index: FromPool<Utf8>,
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
    pub name_index: FromPool<Utf8>,
    /// Entry must be `Utf8`
    pub descriptor_index: FromPool<Utf8>,
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
    Field(FromPool<Fieldref>),
    Method(FromPool<MethodRef>),
    Interface(FromPool<InterfaceMethodref>),
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MethodType {
    /// Entry must be `Utf8`
    pub descriptor_index: FromPool<Utf8>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct InvokeDynamic {
    /// Must be a valid index into the `bootstrap_methods` array of the bootstrap method table of this class field
    pub bootstrap_method_attr_index: u2,
    /// Entry must `NameAndType`
    pub name_and_type_index: FromPool<NameAndType>,
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

impl ValidateCpInfo for Utf8 {
    fn validate_cp_info(info: &[CpInfo], index: u2) -> Result<(), ParseErr> {
        if index == 0 {
            return Err(ParseErr("Index must not be 0".to_string()));
        }
        match &info[index as usize - 1].inner {
            CpInfoInner::Utf8(_) => Ok(()),
            kind => Err(ParseErr(format!(
                concat!("Expected '", stringify!($name), "', found '{:?}'"),
                kind
            ))),
        }
    }
}

// custom implementations
impl<'pool> FromCpInfo<'pool> for Utf8 {
    type Target = &'pool str;

    #[inline]
    fn from_cp_info(info: &'pool CpInfo) -> Self::Target {
        match &info.inner {
            CpInfoInner::Utf8(class) => &class.bytes,
            _ => unreachable!(),
        }
    }
}
