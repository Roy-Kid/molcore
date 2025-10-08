//! Minimal C ABI array type for FFI, mirroring kernel's Array concepts.

use super::dtype::mc_dtype_t;

/// C ABI struct for an array descriptor: only shape and dtype.
#[repr(C)]
pub struct mc_array_t {
    pub shape: *const i64,
    pub shape_len: usize,
    pub dtype: mc_dtype_t,
}

/// Opaque pointer to an array descriptor (can be null for placeholder use cases).
pub type mc_array_handle_t = *mut mc_array_t;

/// Create a null array handle (placeholder for simple linkage tests).
#[unsafe(no_mangle)]
pub extern "C" fn mc_array_null() -> mc_array_handle_t {
    core::ptr::null_mut()
}

/// Check if an array handle is null.
#[unsafe(no_mangle)]
pub extern "C" fn mc_array_is_null(handle: mc_array_handle_t) -> bool {
    handle.is_null()
}
