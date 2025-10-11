/// Data type enum, similar to NumPy's dtype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DType {
    /// 8-bit signed integer
    Int8,
    /// 16-bit signed integer
    Int16,
    /// 32-bit signed integer
    Int32,
    /// 64-bit signed integer
    Int64,
    /// 8-bit unsigned integer
    UInt8,
    /// 16-bit unsigned integer
    UInt16,
    /// 32-bit unsigned integer
    UInt32,
    /// 64-bit unsigned integer
    UInt64,
    /// 32-bit floating point
    Float32,
    /// 64-bit floating point
    Float64,
    /// Boolean
    Bool,
}

impl DType {
    /// Returns the size of the data type in bytes
    pub fn size(&self) -> usize {
        match self {
            DType::Int8 | DType::UInt8 | DType::Bool => 1,
            DType::Int16 | DType::UInt16 => 2,
            DType::Int32 | DType::UInt32 | DType::Float32 => 4,
            DType::Int64 | DType::UInt64 | DType::Float64 => 8,
        }
    }

    /// Returns the name of the data type
    pub fn name(&self) -> &'static str {
        match self {
            DType::Int8 => "int8",
            DType::Int16 => "int16",
            DType::Int32 => "int32",
            DType::Int64 => "int64",
            DType::UInt8 => "uint8",
            DType::UInt16 => "uint16",
            DType::UInt32 => "uint32",
            DType::UInt64 => "uint64",
            DType::Float32 => "float32",
            DType::Float64 => "float64",
            DType::Bool => "bool",
        }
    }
}

/// Trait to get the DType of a Rust type
pub trait HasDType {
    /// Returns the DType corresponding to this Rust type
    fn dtype() -> DType;
}

// Implement HasDType for common numeric types
impl HasDType for i8 { fn dtype() -> DType { DType::Int8 } }
impl HasDType for i16 { fn dtype() -> DType { DType::Int16 } }
impl HasDType for i32 { fn dtype() -> DType { DType::Int32 } }
impl HasDType for i64 { fn dtype() -> DType { DType::Int64 } }
impl HasDType for u8 { fn dtype() -> DType { DType::UInt8 } }
impl HasDType for u16 { fn dtype() -> DType { DType::UInt16 } }
impl HasDType for u32 { fn dtype() -> DType { DType::UInt32 } }
impl HasDType for u64 { fn dtype() -> DType { DType::UInt64 } }
impl HasDType for f32 { fn dtype() -> DType { DType::Float32 } }
impl HasDType for f64 { fn dtype() -> DType { DType::Float64 } }
impl HasDType for bool { fn dtype() -> DType { DType::Bool } }