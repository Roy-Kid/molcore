//! C-ABI Vec3 utilities under core::array to mirror kernel layout.

use crate::status::{mc_status_t, MC_STATUS_SUCCESS};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct mc_vec3f32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct mc_vec3f64 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[inline]
fn status_err() -> mc_status_t { 1 }

#[no_mangle]
pub extern "C" fn mc_vec3f32_from_ptr(ptr: *const f32, out: *mut mc_vec3f32) -> mc_status_t {
    if ptr.is_null() || out.is_null() { return status_err(); }
    unsafe {
        *out = mc_vec3f32 { x: *ptr.add(0), y: *ptr.add(1), z: *ptr.add(2) };
    }
    MC_STATUS_SUCCESS
}

#[no_mangle]
pub extern "C" fn mc_vec3f32_to_ptr(v: *const mc_vec3f32, out_ptr: *mut f32, out_len: usize) -> mc_status_t {
    if v.is_null() || out_ptr.is_null() || out_len < 3 { return status_err(); }
    unsafe {
        let v = &*v;
        *out_ptr.add(0) = v.x;
        *out_ptr.add(1) = v.y;
        *out_ptr.add(2) = v.z;
    }
    MC_STATUS_SUCCESS
}

#[no_mangle]
pub extern "C" fn mc_vec3f64_from_ptr(ptr: *const f64, out: *mut mc_vec3f64) -> mc_status_t {
    if ptr.is_null() || out.is_null() { return status_err(); }
    unsafe {
        *out = mc_vec3f64 { x: *ptr.add(0), y: *ptr.add(1), z: *ptr.add(2) };
    }
    MC_STATUS_SUCCESS
}

#[no_mangle]
pub extern "C" fn mc_vec3f64_to_ptr(v: *const mc_vec3f64, out_ptr: *mut f64, out_len: usize) -> mc_status_t {
    if v.is_null() || out_ptr.is_null() || out_len < 3 { return status_err(); }
    unsafe {
        let v = &*v;
        *out_ptr.add(0) = v.x;
        *out_ptr.add(1) = v.y;
        *out_ptr.add(2) = v.z;
    }
    MC_STATUS_SUCCESS
}
