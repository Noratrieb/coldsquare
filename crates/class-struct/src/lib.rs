#![allow(dead_code)]

struct MethodSignature {
    args: Vec<Type>,
    return_t: Type,
}

/// A Java type, found in signatures
enum Type {
    /// V
    Void,
    /// B
    Boolean,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Object,
    /// [
    Array(Box<Type>),
}
