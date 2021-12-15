//!
//! The models for a .class file
//!
//! [The .class specs](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html)
//!
//! todo poart to [SE16](https://docs.oracle.com/javase/specs/jvms/se16/html/jvms-4.html)
#![allow(dead_code)]

/// All of the Constants in the Constant Pool
pub mod cp_info;

pub use cp_info::FromPool;

// The types used in the specs
#[allow(non_camel_case_types)]
pub type u1 = u8;
#[allow(non_camel_case_types)]
pub type u2 = u16;
#[allow(non_camel_case_types)]
pub type u4 = u32;

///
/// # Represents a .class file
///
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ClassFile {
    /// Magic number identifying the format (= 0xCAFEBABE)
    pub magic: u4,
    /// The version of the class file (.X)
    pub minor_version: u2,
    /// The version of the class file (X.)
    pub major_version: u2,
    /// `constant_pool_count` = Number of entries in the constant pool + 1  
    /// The constant pool. Indexed from 1 to constant_pool_count - 1
    pub constant_pool: Vec<CpInfo>,
    /// Mask of `ClassAccessFlag` used to denote access permissions
    pub access_flags: u2,
    /// A valid index into the `constant_pool` table. The entry must be a `Class`
    pub this_class: FromPool<cp_info::Class>,
    /// Zero or a valid index into the `constant_pool` table
    pub super_class: FromPool<Option<cp_info::Class>>,
    /// Each entry must be a valid index into the `constant_pool` table. The entry must be a `Class`
    pub interfaces: Vec<FromPool<cp_info::Class>>,
    /// All fields of the class. Contains only fields of the class itself
    pub fields: Vec<FieldInfo>,
    /// All methods of the class. If it's neither Native nor Abstract, the implementation has to be provided too
    pub methods: Vec<MethodInfo>,
    /// All attributes of the class
    pub attributes: Vec<AttributeInfo>,
}

/// A constant from the constant pool
/// May have indices back to the constant pool, with expected types
/// _index: A valid index into the `constant_pool` table.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CpInfo {
    pub tag: u1,
    pub inner: CpInfoInner,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CpInfoInner {
    Class(cp_info::Class),
    Fieldref(cp_info::Fieldref),
    MethodRef(cp_info::MethodRef),
    InterfaceMethodref(cp_info::InterfaceMethodref),
    String(cp_info::String),
    Integer(cp_info::Integer),
    Float(cp_info::Float),
    /// 8 byte constants take up two spaces in the constant pool
    Long(cp_info::Long),
    /// 8 byte constants take up two spaces in the constant pool
    Double(cp_info::Double),
    /// Any field or method, without the class it belongs to
    NameAndType(cp_info::NameAndType),
    Utf8(cp_info::Utf8),
    MethodHandle(cp_info::MethodHandle),
    MethodType(cp_info::MethodType),
    Dynamic(cp_info::Dynamic),
    InvokeDynamic(cp_info::InvokeDynamic),
    Module(cp_info::Module),
    Package(cp_info::Package),
}

/// Information about a field
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FieldInfo {
    /// Mask of `FieldAccessFlag` used to denote access permissions
    pub access_flags: u2,
    /// Entry must be `Utf8`
    pub name_index: FromPool<cp_info::Utf8>,
    /// Entry must be `Utf8`
    pub descriptor_index: FromPool<cp_info::Utf8>,
    pub attributes: Vec<AttributeInfo>,
}

/// Information about a method
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MethodInfo {
    /// Mask of `MethodAccessFlag` used to denote access permissions
    pub access_flags: u2,
    /// Index to the `constant_pool` of the method name, must be `Utf8`
    pub name_index: FromPool<cp_info::Utf8>,
    /// Index to the `constant_pool` of the method descriptor, must be `Utf8`
    pub descriptor_index: FromPool<cp_info::Utf8>,
    /// The attributes for this method
    pub attributes: Vec<AttributeInfo>,
}

/// Information about an attribute
///  
/// `attribute_name_index`: Index to the `constant_pool`, must be `Utf8`  
/// `attribute_length`: The length of the subsequent bytes, does not include the first 6
///
/// _index: Index to the `constant_pool` table of any type
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AttributeInfo {
    pub attribute_name_index: FromPool<cp_info::Utf8>,
    pub attribute_length: u4,
    /// The attribute value
    pub inner: AttributeInfoInner,
}

/// The Attributes, without the two common fields
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AttributeInfoInner {
    __Empty,
    /// The exact kind of attribute is not known yet and will be resolved later in the process
    Unknown {
        attribute_content: Vec<u1>,
    },
    /// Only on fields, the constant value of that field
    ConstantValue {
        /// Must be of type `Long`/`Float`/`Double`/`Integer`/`String`
        constantvalue_index: FromPool<CpInfoInner>,
    },
    /// Only on methods, contains JVM instructions and auxiliary information for a single method
    Code {
        /// The maximum depth of the operand stack for this method
        max_stack: u2,
        /// The number of the local variables array, including the parameters
        max_locals: u2,
        /// The JVM bytecode of this method
        code: Vec<u1>,
        /// The exception handlers for this method
        exception_table: Vec<AttributeCodeException>,
        /// The attributes of the code
        attributes: Vec<AttributeInfo>,
    },
    /// Only on the `Code` attribute, used for verification
    /// May be implicit on version >= 50.0, with no entries
    StackMapTable {
        number_of_entries: u2,
        entries: Vec<StackMapFrame>,
    },
    /// Only on `MethodInfo`, indicates which checked exceptions might be thrown
    Exceptions {
        /// Must be a `Class` constant
        exception_index_table: Vec<u2>,
    },
    /// Only on a `ClassFile`. Specifies the inner classes of a class
    InnerClasses {
        classes: Vec<AttributeInnerClass>,
    },
    /// Only on a `ClassFile`, required if it is local or anonymous
    EnclosingMethod {
        /// Must be a `Class` constant, the innermost enclosing class
        class_index: FromPool<cp_info::Class>,
        /// Must be zero or `NameAndType`
        method_index: FromPool<cp_info::NameAndType>,
    },
    /// Can be on `ClassFile`, `FieldInfo`,or `MethodInfo`.
    /// Every generated class has to have this attribute or the `Synthetic` Accessor modifier
    Synthetic,
    /// Can be on `ClassFile`, `FieldInfo`,or `MethodInfo`. Records generic signature information
    Signature {
        /// Must be `Utf8`, and a Class/Method/Field signature
        signature_index: FromPool<cp_info::Utf8>,
    },
    /// Only on a `ClassFile`
    SourceFile {
        /// Must be `Utf8`, the name of the source filed
        sourcefile_index: FromPool<cp_info::Utf8>,
    },
    /// Only on a `ClassFile`
    SourceDebugExtension {
        /// A modified UTF-8 of additional debugging information, `attribute_length`: number of items in `debug_extension`
        debug_extension: Vec<u1>,
    },
    /// Only on the `Code` attribute. It includes line number information used by debuggers
    LineNumberTable {
        line_number_table: Vec<AttributeLineNumber>,
    },
    /// Only on the `Code` attribute. It may be used to determine the value of local variables by debuggers
    LocalVariableTable {
        /// Note: the 3rd field is called `descriptor_index` and represents an field descriptor
        local_variable_table: Vec<AttributeLocalVariableTable>,
    },
    /// Only on the `Code` attribute. It provides signature information instead of descriptor information
    LocalVariableTypeTable {
        /// Note: the 3rd field is called `signature_index` and represents a field type signature
        local_variable_table: Vec<AttributeLocalVariableTable>,
    },
    /// Can be on `ClassFile`, `FieldInfo`,or `MethodInfo`. Marks a class/field/method as deprecated
    Deprecated,
    /// Can be on `ClassFile`, `FieldInfo`,or `MethodInfo`. Contains all Runtime visible annotations
    RuntimeVisibleAnnotations {
        annotations: Vec<Annotation>,
    },
    /// Same as `RuntimeVisibleAnnotations`, but invisible to reflection
    RuntimeInvisibleAnnotations {
        annotations: Vec<Annotation>,
    },
    /// Only on `MethodInfo`, parameter annotations visible during runtime
    RuntimeVisibleParameterAnnotations {
        parameter_annotations: Vec<ParameterAnnotation>,
    },
    /// Same as `RuntimeVisibleParameterAnnotations`, but invisible to reflection
    RuntimeInvisibleParameterAnnotations {
        parameter_annotations: Vec<ParameterAnnotation>,
    },
    /// Only on `MethodInfo`, on those representing elements of annotation types, the default value of the element
    AnnotationDefault {
        default_value: AnnotationElementValue,
    },
    /// Only on `ClassFile`. Records bootstrap method specifiers for `invokedynamic`
    BootstrapMethods {
        bootstrap_methods: Vec<BootstrapMethod>,
    },
    /// Only on `ClassFile`, where there may be one at most. Specifies packages exported and opened by a module
    Module(Box<Module>),

    // todo
    MethodParameters,
    ModulePackages,
    ModuleMainClass,
    NestHost,
    NestMembers,
    Record,
}

/// An exception handler in the JVM bytecode array
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AttributeCodeException {
    /// The ranges in the code in which the handler is active. Must be a valid index into the code array.
    /// The `start_pc` is inclusive
    pub start_pc: u2,
    /// The ranges in the code in which the handler is active. Must be a valid index into the code array or the length.
    /// The `end_pc` is exclusive
    pub end_pc: u2,
    /// The start of the exception handler, must be a valid index into the code array at an opcode instruction
    pub handler_pc: u2,
    /// If the catch type is nonzero, it must be a valid index into the `constant_pool`, must be a `Class`
    /// Zero means it catches all Exceptions, this is usually for `finally`
    pub catch_type: u2,
}

/// Specifies the type state at a particular bytecode offset
/// Has a offset_delta, the offset is calculated by adding offset_delta + 1 to the previous offset
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StackMapFrame {
    /// Exactly the same locals as the previous frame and zero stack items, offset_delta is frame_type
    SameFrame {
        frame_type: u1, // 0-63
    },
    /// Exactly the same locals as the previous frame and 1 stack item, offset_delta is (frame_type - 64)
    SameLocals1StackItemFrame {
        frame_type: u1, // 64-127
        stack: VerificationTypeInfo,
    },
    /// Exactly the same locals as the previous frame and 1 stack item, offset_delta is given explicitly
    SameLocals1StackItemFrameExtended {
        frame_type: u1, // 247
        offset_delta: u2,
        stack: VerificationTypeInfo,
    },
    /// Operand stack is empty and the locals are the same, except for the *k* last locals (`k = 251 - frame_type`)
    ChopFrame {
        frame_type: u1, // 248-250
        offset_delta: u2,
    },
    /// Exactly the same locals as the previous frame and zero stack items
    SameFrameExtended {
        frame_type: u1, // 251
        offset_delta: u2,
    },
    /// Operand stack is empty and exactly the same locals as the previous frame, except k locals are added, (`k = frame_type-251`)
    AppendFrame {
        frame_type: u1, // 252-254
        offset_delta: u2,
        /// `length = frame_type - 251`
        locals: Vec<VerificationTypeInfo>,
    },
    /// The stack or Variable entries in the locals/stack can be either 1 or 2 entries wide, depending on the type
    FullFrame {
        frame_type: u1, //255
        offset_delta: u2,
        locals: Vec<VerificationTypeInfo>,
        stack: Vec<VerificationTypeInfo>,
    },
}

/// A stack value/local variable type `StackMapFrame`
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum VerificationTypeInfo {
    Top {
        tag: u1, // 0
    },
    Integer {
        tag: u1, // 1
    },
    Float {
        tag: u1, // 2
    },
    Long {
        tag: u1, // 4
    },
    Double {
        tag: u1, // 3
    },
    Null {
        tag: u1, // 5
    },
    UninitializedThis {
        tag: u1, // 6
    },
    Object {
        tag: u1, // 7
        /// Must be a `Class`
        cpool_index: FromPool<cp_info::Class>,
    },
    Uninitialized {
        tag: u1, // 8
        offset: u2,
    },
}

/// A struct for the `AttributeInfo::InnerClasses`
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AttributeInnerClass {
    /// Must be a `Class`
    pub inner_class_info_index: FromPool<cp_info::Class>,
    /// Must be 0 or a `Class`
    pub outer_class_info_index: FromPool<cp_info::Class>,
    /// Must be 0 or `Utf8`
    pub inner_class_name_index: FromPool<cp_info::Utf8>,
    /// Must be a mask of `InnerClassAccessFlags`
    pub inner_class_access_flags: u2,
}

/// Line number information for `AttributeInfo::LineNumberTable`
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AttributeLineNumber {
    /// Index into the code array where a new line in the source begins
    pub start_pc: u2,
    /// The line number in the source file
    pub line_number: u2,
}

/// Local variable information for `AttributeInfo::LocalVariableTable` and `AttributeInfo::LocalVariableTypeTable`
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AttributeLocalVariableTable {
    /// The local variable must have a value between `start_pc` and `start_pc + length`. Must be a valid opcode
    pub start_pc: u2,
    /// The local variable must have a value between `start_pc` and `start_pc + length`
    pub length: u2,
    /// Must be `Utf8`
    pub name_index: FromPool<cp_info::Utf8>,
    /// Must be `Utf8`, field descriptor or field signature encoding the type
    pub descriptor_or_signature_index: FromPool<cp_info::Utf8>,
    /// The variable must be at `index` in the local variable array
    pub index: u2,
}

/// A runtime-visible annotation to the program
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Annotation {
    /// Must be `Utf8`
    pub type_index: FromPool<cp_info::Utf8>,
    pub num_element_value_pairs: u2,
    pub element_value_pairs: Vec<AnnotationElementValuePair>,
}

// these type names have just become java at this point. no shame.

/// A element-value pair in the `Annotation`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AnnotationElementValuePair {
    /// Must be `Utf8`
    pub element_name_index: FromPool<cp_info::Utf8>,
    pub element_name_name: AnnotationElementValue,
}

/// The value of an `AnnotationElementValuePair`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AnnotationElementValue {
    /// B, C, D, F, I, J, S, Z or s, e, c, @,
    pub tag: u1,
    pub value: AnnotationElementValueValue,
}

/// The value of a `AnnotationElementValue`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AnnotationElementValueValue {
    /// If the tag is B, C, D, F, I, J, S, Z, or s.
    ConstValueIndex {
        /// Must be the matching constant pool entry
        index: FromPool<CpInfoInner>,
    },
    /// If the tag is e
    EnumConstValue {
        /// Must be `Utf8`
        type_name_index: FromPool<cp_info::Utf8>,
        /// Must be `Utf8`
        const_name_index: FromPool<cp_info::Utf8>,
    },
    /// If the tag is c
    ClassInfoIndex {
        /// Must be `Utf8`, for example Ljava/lang/Object; for Object
        index: FromPool<cp_info::Utf8>,
    },
    /// If the tag is @
    AnnotationValue {
        /// Represents a nested annotation
        annotation: Box<Annotation>,
    },
    /// If the tag is [
    ArrayValue { values: Vec<AnnotationElementValue> },
}

/// Used in `AttributeInfo::RuntimeVisibleParameterAnnotations`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ParameterAnnotation {
    pub annotations: Vec<Annotation>,
}

/// Used in `AttributeInfo::BootstrapMethods `
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BootstrapMethod {
    /// Must be a `MethodHandle`
    pub bootstrap_method_ref: FromPool<cp_info::MethodHandle>,
    /// Each argument is a cpool entry. The constants must be `String, Class, Integer, Long, Float, Double, MethodHandle, or MethodType`
    pub bootstrap_arguments: Vec<FromPool<CpInfoInner>>,
}

/// Used in `AttributeInfo::Module`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Module {
    pub module_name_index: FromPool<cp_info::Utf8>,
    /// The following flags exist
    /// * 0x0020 (ACC_OPEN) - Indicates that this module is open.
    /// * 0x1000 (ACC_SYNTHETIC) - Indicates that this module was not explicitly or implicitly declared.
    /// * 0x8000 (ACC_MANDATED) - Indicates that this module was implicitly declared.
    pub module_flags: u2,
    /// The version of the module
    pub module_version_index: FromPool<Option<cp_info::Utf8>>,
    /// If the module is `java.base`, the Vec must be empty
    pub requires: Vec<ModuleRequires>,
    pub exports: Vec<ModuleExports>,
    pub opens: Vec<ModuleOpens>,
    pub uses_index: Vec<u2>,
    pub provides: Vec<ModuleProvides>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModuleRequires {
    pub requires_index: FromPool<cp_info::Module>,
    /// * 0x0020 (ACC_TRANSITIVE) - Indicates that any module which depends on the current module, implicitly declares a dependence on the module indicated by this entry.
    /// * 0x0040 (ACC_STATIC_PHASE) - Indicates that this dependence is mandatory in the static phase, i.e., at compile time, but is optional in the dynamic phase, i.e., at run time.
    /// * 0x1000 (ACC_SYNTHETIC) - Indicates that this dependence was not explicitly or implicitly declared in the source of the module declaration.
    /// * 0x8000 (ACC_MANDATED) - Indicates that this dependence was implicitly declared in the source of the module declaration.
    /// If the current module is not java.base, and the class file version number is 54.0 or above, then neither ACC_TRANSITIVE nor ACC_STATIC_PHASE may be set in requires_flags.
    pub requires_flags: u2,
    pub requires_version_index: FromPool<Option<cp_info::Utf8>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModuleExports {
    pub exports_index: FromPool<cp_info::Package>,
    /// * 0x1000 (ACC_SYNTHETIC) - Indicates that this export was not explicitly or implicitly declared in the source of the module declaration.
    /// * 0x8000 (ACC_MANDATED) - Indicates that this export was implicitly declared in the source of the module declaration.
    pub exports_flags: u2,
    /// If there are no exports, the package is *unqualified*, allowing unrestricted access  
    /// If there are exports, the package is *qualified*, only allowing the following modules can access it
    pub exports_to_index: Vec<FromPool<cp_info::Module>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModuleOpens {
    pub opens_index: FromPool<cp_info::Module>,
    /// * 0x1000 (ACC_SYNTHETIC) - Indicates that this opening was not explicitly or implicitly declared in the source of the module declaration.
    /// * 0x8000 (ACC_MANDATED) - Indicates that this opening was implicitly declared in the source of the module declaration.
    pub opens_flags: u2,
    /// If there are no exports, the package is *unqualified*, allowing unrestricted reflective access  
    /// If there are exports, the package is *qualified*, only allowing the following modules can reflectively access it
    pub opens_to_index: Vec<FromPool<cp_info::Module>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A service interface for which this module represents an implementation
pub struct ModuleProvides {
    /// Represents the interface
    pub provides_index: FromPool<cp_info::Class>,
    /// Represents the implementations, must be nonzero
    pub provides_with_index: Vec<FromPool<cp_info::Class>>,
}

/////// Access Flags

/// Access Flags of a class
#[repr(u16)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ClassAccessFlag {
    /// Declared public; may be accessed from outside its package.
    Public = 0x0001,
    /// Declared final; no subclasses allowed.
    Final = 0x0010,
    /// Treat superclass methods specially when invoked by the invokespecial instruction.
    Super = 0x0020,
    /// Is an interface, not a class.
    Interface = 0x0200,
    /// Declared abstract; must not be instantiated.
    Abstract = 0x0400,
    /// Declared synthetic; not present in the source code.
    Synthetic = 0x1000,
    /// Declared as an annotation type.
    Annotation = 0x2000,
    /// Declared as an enum type.
    Enum = 0x4000,
    /// Is a module, not a class or interface.
    MODULE = 0x8000,
}

/// Access Flags of a method
#[repr(u16)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MethodAccessFlag {
    /// Declared public; may be accessed from outside its package.
    PUBLIC = 0x0001,
    /// Declared private; accessible only within the defining class.
    PRIVATE = 0x0002,
    /// Declared protected; may be accessed within subclasses.
    PROTECTED = 0x0004,
    /// Declared static.
    STATIC = 0x0008,
    /// Declared final; must not be overridden.
    FINAL = 0x0010,
    /// Declared synchronized; invocation is wrapped by a monitor use.
    SYNCHRONIZED = 0x0020,
    /// A bridge method, generated by the compiler.
    BRIDGE = 0x0040,
    /// Declared with variable number of arguments.
    VARARGS = 0x0080,
    /// Declared native; implemented in a language other than Java.
    NATIVE = 0x0100,
    /// Declared abstract; no implementation is provided.
    ABSTRACT = 0x0400,
    /// Declared strictfp; floating-point mode is FP-strict.
    STRICT = 0x0800,
    // /Declared synthetic; not present in the source code.
    SYNTHETIC = 0x1000,
}

/// Access flags for an inner class
#[repr(u16)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum InnerClassAccessFlags {
    /// Marked or implicitly public in source.
    PUBLIC = 0x0001,
    /// Marked private in source.
    PRIVATE = 0x0002,
    /// Marked protected in source.
    PROTECTED = 0x0004,
    /// Marked or implicitly static in source.
    STATIC = 0x0008,
    /// Marked final in source.
    FINAL = 0x0010,
    /// Was an interface in source.
    INTERFACE = 0x0200,
    /// Marked or implicitly abstract in source.
    ABSTRACT = 0x0400,
    /// Declared synthetic; not present in the source code.
    SYNTHETIC = 0x1000,
    /// Declared as an annotation type.
    ANNOTATION = 0x2000,
    /// Declared as an enum type.
    ENUM = 0x4000,
}

/// Access flags for a field
#[repr(u16)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FieldAccessFlags {
    /// Declared public; may be accessed from outside its package.
    PUBLIC = 0x0001,
    /// Declared private; usable only within the defining class.
    PRIVATE = 0x0002,
    /// Declared protected; may be accessed within subclasses.
    PROTECTED = 0x0004,
    /// Declared static.
    STATIC = 0x0008,
    /// Declared final; never directly assigned to after object construction (JLS §17.5).
    FINAL = 0x0010,
    /// Declared volatile; cannot be cached.
    VOLATILE = 0x0040,
    /// Declared transient; not written or read by a persistent object manager.
    TRANSIENT = 0x0080,
    /// Declared synthetic; not present in the source code.
    SYNTHETIC = 0x1000,
    /// Declared as an element of an enum.
    ENUM = 0x4000,
}
