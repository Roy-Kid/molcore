pub mod status;
pub mod core;

#[unsafe(no_mangle)]
pub extern "C" fn mc_version() -> u32 {
    1
}
