//! Minimal C ABI status type and helpers for the FFI surface.
//! Keep it intentionally tiny for design validation only.

use core::ffi::c_char;

/// Status code used in the C API.
/// 0 means success, any non-zero value is an error code.
pub type mc_status_t = i32;

/// Constant success status.
pub const MC_STATUS_SUCCESS: mc_status_t = 0;

/// Create a non-zero error status from an integer code.
/// Caller is responsible to ensure `code != 0`.
#[unsafe(no_mangle)]
pub extern "C" fn mc_status_error(code: mc_status_t) -> mc_status_t {
	if code == 0 { 1 } else { code }
}

/// Convenience: convert a boolean (ok?) to status.
#[unsafe(no_mangle)]
pub extern "C" fn mc_status_from_bool(ok: bool) -> mc_status_t {
	if ok { MC_STATUS_SUCCESS } else { 1 }
}

/// Optional: map a status to a static message pointer for debugging only.
/// Returned pointer is valid for the duration of the program and must NOT be freed.
#[unsafe(no_mangle)]
pub extern "C" fn mc_status_message(status: mc_status_t) -> *const c_char {
	match status {
		0 => b"OK\0".as_ptr() as *const c_char,
		1 => b"ERROR\0".as_ptr() as *const c_char,
		_ => b"ERROR\0".as_ptr() as *const c_char,
	}
}
