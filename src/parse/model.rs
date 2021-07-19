//!
//! The models for a .class file
//!
//! [The .class specs](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html)

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
pub struct ClassFile {
    /// Magic number identifying the format (= 0xCAFEBABE)
    pub(crate) magic: u4,
    /// The version of the class file (.X)
    pub(crate) minor_version: u2,
    /// The version of the class file (X.)
    pub(crate) major_version: u2,
    /// Number of entries in the constant pool + 1
    pub(crate) constant_pool_count: u2,
    /// The constant pool. Indexed from 1 to constant_pool_count - 1
    pub(crate) constant_pool: Vec<CpInfo>,
    /// Mask of `ClassAccessFlag` used to denote access permissions
    pub(crate) access_flags: u2,
    /// A valid index into the `constant_pool` table. The entry must be a `ConstantClassInfo`
    pub(crate) this_class: u2,
    /// Zero or a valid index into the `constant_pool` table
    pub(crate) super_class: u2,
    /// The number if direct superinterfaces of this class or interface type
    pub(crate) interfaces_count: u2,
    /// Each entry must be a valid index into the `constant_pool` table. The entry must be a `ConstantClassInfo`
    pub(crate) interfaces: Vec<u2>,
    /// The number of fields in the `fields` table
    pub(crate) fields_count: u2,
    /// All fields of the class. Contains only fields of the class itself
    pub(crate) fields: Vec<FieldInfo>,
    /// The number of methods in `methods`
    pub(crate) method_count: u2,
    /// All methods of the class. If it's neither Native nor Abstract, the implementation has to be provided too
    pub(crate) methods: Vec<MethodInfo>,
    /// The number of attributes in `attributes`
    pub(crate) attributes_count: u2,
    /// All attributes of the class
    pub(crate) attributes: Vec<AttributeInfo>,
}

/// A constant from the constant pool
/// May have indices back to the constant pool, with expected types
/// _index: A valid index into the `constant_pool` table.
pub enum CpInfo {
    Class {
        tag: u1, // 7
        /// Entry must be `Utf8`
        name_index: u2,
    },
    Fieldref {
        tag: u1, // 9
        /// May be a class or interface type
        class_index: u2,
        /// Entry must be `NameAndType`
        name_and_type_index: u2,
    },
    Methodref {
        tag: u1, // 10
        /// Must be a class type
        class_index: u2,
        /// Entry must be `NameAndType`
        name_and_type_index: u2,
    },
    InterfaceMethodref {
        tag: u1, // 11
        /// Must be an interface type
        class_index: u2,
        /// Entry must be `NameAndType`
        name_and_type_index: u2,
    },
    String {
        tag: u1, // 8
        /// Entry must be `Utf8`
        string_index: u2,
    },
    Integer {
        tag: u1, // 3
        // Big endian
        bytes: u4,
    },
    Float {
        tag: u1, // 4
        /// IEEE 754 floating-point single format, big endian
        bytes: u4,
    },
    /// 8 byte constants take up two spaces in the constant pool
    Long {
        tag: u1, // 5
        /// Big endian
        high_bytes: u4,
        /// Big endian
        low_bytes: u4,
    },
    /// 8 byte constants take up two spaces in the constant pool
    Double {
        tag: u1, // 6
        /// IEEE 754 floating-point double format, big endian
        high_bytes: u4,
        /// IEEE 754 floating-point double format, big endian
        low_bytes: u4,
    },
    /// Any field or method, without the class it belongs to
    NameAndType {
        tag: u1, // 12
        /// Entry must be `Utf8`
        name_index: u2,
        /// Entry must be `Utf8`
        descriptor_index: u2,
    },
    Utf8 {
        tag: u1, // 1
        /// The length of the String. Not null-terminated.
        length: u2,
        /// Contains modified UTF-8
        bytes: Vec<u1>,
    },
    MethodHandle {
        tag: u1, // 15
        /// The kind of method handle (0-9)
        reference_kind: u1,
        /// If the kind is 1-4, the entry must be `FieldRef`. If the kind is 5-8, the entry must be `MethodRef`
        /// If the kind is 9, the entry must be `InterfaceMethodRef`
        reference_index: u2,
    },
    MethodType {
        tag: u1, // 16
        /// Entry must be `Utf8`
        descriptor_index: u2,
    },
    InvokeDynamic {
        tag: u1,
        /// Must be a valid index into the `bootstrap_methods` array of the bootstrap method table of this class fiel
        bootstrap_method_attr_index: u2,
        /// Entry must `NameAndType`
        name_and_type_index: u2,
    },
}

/// Information about a field
pub struct FieldInfo {
    access_flags: u2,
    name_index: u2,
    descriptor_index: u2,
    attributes_count: u2,
    attributes: Vec<AttributeInfo>,
}

/// Information about a method
pub struct MethodInfo {
    /// Mask of `MethodAccessFlag` used to denote access permissions
    access_flags: u2,
    /// Index to the `constant_pool` of the method name, must be `Utf8`
    name_index: u2,
    /// Index to the `constant_pool` of the method descriptor, must be `Utf8`
    descriptor_index: u2,
    /// The amount of attributes for this method
    attributes_count: u2,
    /// The attributes for this method
    attributes: Vec<AttributeInfo>,
}

/// Information about an attribute
///  
/// `attribute_name_index`: Index to the `constant_pool`, must be `Utf8`  
/// `attribute_length`: The length of the subsequent bytes, does not include the first 6
///
/// _index: Index to the `constant_pool` table of any type
pub enum AttributeInfo {
    /// Only on fields, the constant value of that field
    ConstantValue {
        attribute_name_index: u2, // "ConstantValue"
        attribute_length: u4,
        /// Must be of type `Long`/`Float`/`Double`/`Integer`/`String`
        constantvalue_index: u2,
    },
    /// Only on methods, contains JVM instructions and auxiliary information for a single method
    Code {
        attribute_name_index: u2,
        attribute_length: u4,
        /// The maximum depth of the operand stack for this method
        max_stack: u2,
        /// The number of the local variables array, including the parameters
        max_locals: u2,
        /// The length of the JVM bytecode in bytes
        code_length: u4,
        /// The JVM bytecode of this method
        code: Vec<u1>,
        /// The number of entries in the exception table
        exception_table_length: u2,
        /// The exception handlers for this method
        exception_table: Vec<AttributeCodeException>,
        attributes_count: u2,
        /// The attributes of the code
        attributes: Vec<AttributeInfo>,
    },
    /// Only on the `Code` attribute, used for verification
    /// May be implicit on version >= 50.0, with no entries
    StackMapTable {
        /// Must be `Utf8`
        attribute_name_index: u2,
        attribute_length: u4,
        number_of_entries: u2,
        entries: Vec<StackMapFrame>,
    },
    /// Only on `MethodInfo`, indicates which checked exceptions might be thrown
    Exceptions {
        attribute_name_index: u2,
        attribute_length: u4,
        number_of_exceptions: u2,
        /// Must be a `Class` constant
        exception_index_table: Vec<u2>,
    },
    /// Only on a `ClassFile`. Specifies the inner classes of a class
    InnerClasses {
        attribute_name_index: u2,
        attribute_length: u4,
        number_of_classes: u2,
        classes: Vec<AttributeInnerClass>,
    },
    /// Only on a `ClassFile`, required if it is local or anonymous
    EnclosingMethod {
        attribute_name_index: u2,
        attribute_length: u4, // 4
        /// Must be a `Class` constant, the innermost enclosing class
        class_index: u2,
        /// Must be zero or `NameAndType`
        method_index: u2,
    },
    /// Can be on `ClassFile`, `FieldInfo`,or `MethodInfo`.
    /// Every generated class has to have this attribute or the `Synthetic` Accessor modifier
    Synthetic {
        attribute_name_index: u2,
        attribute_length: u4, // 0
    },
    /// Can be on `ClassFile`, `FieldInfo`,or `MethodInfo`. Records generic signature information
    Signature {
        attribute_name_index: u2,
        attribute_length: u4, // 2
        /// Must be `Utf8`, and a Class/Method/Field signature
        signature_index: u2,
    },
    /// Only on a `ClassFile`
    SourceFile {
        attribute_name_index: u2,
        attribute_length: u4, // 2
        /// Must be `Utf8`, the name of the source filed
        sourcefile_index: u2,
    },
    /// Only on a `ClassFile`
    SourceDebugExtension {
        attribute_name_index: u2,
        attribute_length: u4, // number of items in `debug_extension`
        /// A modified UTF-8 of additional debugging information
        debug_extension: Vec<u1>,
    },
    /// Only on the `Code` attribute. It includes line number information used by debuggers
    LineNumberTable {
        attribute_name_index: u2,
        attribute_length: u4,
        line_number_table_length: u2,
        line_number_table: Vec<AttributeLineNumber>,
    },
    /// Only on the `Code` attribute. It may be used to determine the value of local variables by debuggers
    LocalVariableTable {
        attribute_name_index: u2,
        attribute_length: u4,
        local_variable_table_length: u2,
        /// Note: the 3rd field is called `descriptor_index` and represents an field descriptor
        local_variable_table: Vec<AttributeLocalVariableTable>,
    },
    /// Only on the `Code` attribute. It provides signature information instead of descriptor information
    LocalVariableTypeTable {
        attribute_name_index: u2,
        attribute_length: u4,
        local_variable_table_length: u2,
        /// Note: the 3rd field is called `signature_index` and represents a field type signature
        local_variable_table: Vec<AttributeLocalVariableTable>,
    },
    /// Can be on `ClassFile`, `FieldInfo`,or `MethodInfo`. Marks a class/field/method as deprecated
    Deprecated {
        attribute_name_index: u2,
        attribute_length: u4, // 0
    },
    /// Can be on `ClassFile`, `FieldInfo`,or `MethodInfo`. Contains all Runtime visible annotations
    RuntimeVisibleAnnotations {
        attribute_name_index: u2,
        attribute_length: u4,
        num_annotations: u2,
        annotations: Vec<Annotation>,
    },
    /// Same as `RuntimeVisibleAnnotations`, but invisible to reflection
    RuntimeInvisibleAnnotations {
        attribute_name_index: u2,
        attribute_length: u4,
        num_annotations: u2,
        annotations: Vec<Annotation>,
    },
    /// Only on `MethodInfo`, parameter annotations visible during runtime
    RuntimeVisibleParameterAnnotations {
        attribute_name_index: u2,
        attribute_length: u4,
        num_parameters: u1,
        parameter_annotations: Vec<ParameterAnnotation>,
    },
    /// Same as `RuntimeVisibleParameterAnnotations`, but invisible to reflection
    RuntimeInvisibleParameterAnnotations {
        attribute_name_index: u2,
        attribute_length: u4,
        num_parameters: u1,
        parameter_annotations: Vec<ParameterAnnotation>,
    },
    /// Only on `MethodInfo`, on those representing elements of annotation types, the default value of the element
    AnnotationDefault {
        attribute_name_index: u2,
        attribute_length: u4,
        default_value: AnnotationElementValue,
    },
    /// Only on `ClassFile`. Records bootstrap method specifiers for `invokedynamic`
    BootstrapMethods {
        attribute_name_index: u2,
        attribute_length: u4,
        num_bootstrap_methods: u2,
        bootstrap_methods: Vec<BootstrapMethod>,
    },
}

/// An exception handler in the JVM bytecode array
pub struct AttributeCodeException {
    /// The ranges in the code in which the handler is active. Must be a valid index into the code array.
    /// The `start_pc` is inclusive
    start_pc: u2,
    /// The ranges in the code in which the handler is active. Must be a valid index into the code array or the length.
    /// The `end_pc` is exclusive
    end_pc: u2,
    /// The start of the exception handler, must be a valid index into the code array at an opcode instruction
    handler_pc: u2,
    /// If the catch type is nonzero, it must be a valid index into the `constant_pool`, must be a `Class`
    /// Zero means it catches all Exceptions, this is usually for `finally`
    catch_type: u2,
}

/// Specifies the type state at a particular bytecode offset
/// Has a offset_delta, the offset is calculated by adding offset_delta + 1 to the previous offset
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
        frame_type: u1, // 257
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
        number_of_locals: u2,
        locals: Vec<VerificationTypeInfo>,
        number_of_stack_items: u2,
        stack: Vec<VerificationTypeInfo>,
    },
}

/// A stack value/local variable type `StackMapFrame`
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
        cpool_index: u2,
    },
    Uninitialized {
        tag: u1, // 8
        offset: u2,
    },
}

/// A struct for the `AttributeInfo::InnerClasses`
pub struct AttributeInnerClass {
    /// Must be a `Class`
    inner_class_info_index: u2,
    /// Must be 0 or a `Class`
    outer_class_info_index: u2,
    /// Must be 0 or `Utf8`
    inner_class_name_index: u2,
    /// Must be a mask of `InnerClassAccessFlags`
    inner_class_access_flags: u2,
}

/// Line number information for `AttributeInfo::LineNumberTable`
pub struct AttributeLineNumber {
    /// Index into the code array where a new line in the source begins
    start_pc: u2,
    /// The line number in the source file
    line_number: u2,
}

/// Local variable information for `AttributeInfo::LocalVariableTable` and `AttributeInfo::LocalVariableTypeTable`
pub struct AttributeLocalVariableTable {
    /// The local variable must have a value between `start_pc` and `start_pc + length`. Must be a valid opcode
    start_pc: u2,
    /// The local variable must have a value between `start_pc` and `start_pc + length`
    length: u2,
    /// Must be `Utf8`
    name_index: u2,
    /// Must be `Utf8`, field descriptor or field signature encoding the type
    descriptor_or_signature_index: u2,
    /// The variable must be at `index` in the local variable array
    index: u2,
}

/// A runtime-visible annotation to the program
pub struct Annotation {
    /// Must be `Utf8`
    type_index: u2,
    num_element_value_pairs: u2,
    element_value_pairs: Vec<AnnotationElementValuePair>,
}

// these type names have just become java at this point. no shame.

/// A element-value pair in the `Annotation`
pub struct AnnotationElementValuePair {
    /// Must be `Utf8`
    element_name_index: u2,
    element_name_name: AnnotationElementValue,
}

/// The value of an `AnnotationElementValuePair`
pub struct AnnotationElementValue {
    /// B, C, D, F, I, J, S, Z or s, e, c, @,
    tag: u1,
    value: AnnotationElementValueValue,
}

/// The value of a `AnnotationElementValue`
pub enum AnnotationElementValueValue {
    /// If the tag is B, C, D, F, I, J, S, Z, or s.
    ConstValueIndex {
        /// Must be the matching constant pool entry
        index: u2,
    },
    /// If the tag is e
    EnumConstValue {
        /// Must be `Utf8`
        type_name_index: u2,
        /// Must be `Utf8`
        const_name_index: u2,
    },
    /// If the tag is c
    ClassInfoIndex {
        /// Must be `Utf8`, for example Ljava/lang/Object; for Object
        index: u2,
    },
    /// If the tag is @
    AnnotationValue {
        /// Represents a nested annotation
        annotation: Box<Annotation>,
    },
    /// If the tag is [
    ArrayValue {
        num_values: u2,
        values: Vec<AnnotationElementValue>,
    },
}

/// Used in `AttributeInfo::RuntimeVisibleParameterAnnotations`
pub struct ParameterAnnotation {
    num_annotations: u2,
    annotations: Vec<Annotation>,
}

/// Used in `AttributeInfo::BootstrapMethods `
pub struct BootstrapMethod {
    /// Must be a `MethodHandle`
    bootstrap_method_ref: u2,
    num_bootstrap_arguments: u2,
    /// Each argument is a cpool entry. The constants must be `String, Class, Integer, Long, Float, Double, MethodHandle, or MethodType`
    bootstrap_arguments: Vec<u2>,
}

/////// Access Flags

/// Access Flags of a class
#[repr(u16)]
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
}

/// Access Flags of a method
#[repr(u16)]
pub enum MethodAccessFlag {
    //	Declared public; may be accessed from outside its package.
    PUBLIC = 0x0001,
    //	Declared private; accessible only within the defining class.
    PRIVATE = 0x0002,
    //	Declared protected; may be accessed within subclasses.
    PROTECTED = 0x0004,
    //	Declared static.
    STATIC = 0x0008,
    //	Declared final; must not be overridden.
    FINAL = 0x0010,
    //	Declared synchronized; invocation is wrapped by a monitor use.
    SYNCHRONIZED = 0x0020,
    //	A bridge method, generated by the compiler.
    BRIDGE = 0x0040,
    //	Declared with variable number of arguments.
    VARARGS = 0x0080,
    //	Declared native; implemented in a language other than Java.
    NATIVE = 0x0100,
    //	Declared abstract; no implementation is provided.
    ABSTRACT = 0x0400,
    //	Declared strictfp; floating-point mode is FP-strict.
    STRICT = 0x0800,
    //	Declared synthetic; not present in the source code.
    SYNTHETIC = 0x1000,
}

/// Access flags for an inner class
#[repr(u16)]
pub enum InnerClassAccessFlags {
    ///	Marked or implicitly public in source.
    PUBLIC = 0x0001,
    ///	Marked private in source.
    PRIVATE = 0x0002,
    ///	Marked protected in source.
    PROTECTED = 0x0004,
    ///	Marked or implicitly static in source.
    STATIC = 0x0008,
    /// 	Marked final in source.
    FINAL = 0x0010,
    ///	Was an interface in source.
    INTERFACE = 0x0200,
    ///	Marked or implicitly abstract in source.
    ABSTRACT = 0x0400,
    ///	Declared synthetic; not present in the source code.
    SYNTHETIC = 0x1000,
    ///	Declared as an annotation type.
    ANNOTATION = 0x2000,
    /// Declared as an enum type.
    ENUM = 0x4000,
}

/// Access flags for a field
#[repr(u16)]
pub enum FieldAccessFlags {
    ///	Declared public; may be accessed from outside its package.
    PUBLIC = 0x0001,
    ///	Declared private; usable only within the defining class.
    PRIVATE = 0x0002,
    ///	Declared protected; may be accessed within subclasses.
    PROTECTED = 0x0004,
    ///	Declared static.
    STATIC = 0x0008,
    ///	Declared final; never directly assigned to after object construction (JLS §17.5).
    FINAL = 0x0010,
    ///	Declared volatile; cannot be cached.
    VOLATILE = 0x0040,
    ///	Declared transient; not written or read by a persistent object manager.
    TRANSIENT = 0x0080,
    ///	Declared synthetic; not present in the source code.
    SYNTHETIC = 0x1000,
    ///	Declared as an element of an enum.
    ENUM = 0x4000,
}
