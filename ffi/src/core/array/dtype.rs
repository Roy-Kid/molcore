//! C-ABI representation of DType for FFI, aligned with kernel::core::array::DType.

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum mc_dtype_t {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Bool,
}
