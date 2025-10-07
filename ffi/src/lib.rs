#![no_std]

pub mod status;
pub mod core;

/// Simple exported function to verify linking from C.
#[no_mangle]
pub extern "C" fn mc_version() -> u32 {
    1
}
