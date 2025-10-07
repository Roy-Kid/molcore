// Minimal C header for molcore FFI - concept validation only
#ifndef MOLCORE_H
#define MOLCORE_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef int32_t mc_status_t;
enum mc_dtype_t {
    MC_DTYPE_Int8,
    MC_DTYPE_Int16,
    MC_DTYPE_Int32,
    MC_DTYPE_Int64,
    MC_DTYPE_UInt8,
    MC_DTYPE_UInt16,
    MC_DTYPE_UInt32,
    MC_DTYPE_UInt64,
    MC_DTYPE_Float32,
    MC_DTYPE_Float64,
    MC_DTYPE_Bool,
};

typedef struct mc_array_t {
    const int64_t* shape;
    uintptr_t shape_len;
    enum mc_dtype_t dtype;
} mc_array_t;

// exported helpers
uint32_t mc_version(void);
mc_status_t mc_status_error(mc_status_t code);
mc_status_t mc_status_from_bool(bool ok);
const char* mc_status_message(mc_status_t status);

// array helpers
mc_array_t* mc_array_null(void);
bool mc_array_is_null(mc_array_t* handle);

// vec3 helpers (f32 / f64)
typedef struct { float x, y, z; } mc_vec3f32;
typedef struct { double x, y, z; } mc_vec3f64;
mc_status_t mc_vec3f32_from_ptr(const float* ptr, mc_vec3f32* out);
mc_status_t mc_vec3f32_to_ptr(const mc_vec3f32* v, float* out_ptr, uintptr_t out_len);
mc_status_t mc_vec3f64_from_ptr(const double* ptr, mc_vec3f64* out);
mc_status_t mc_vec3f64_to_ptr(const mc_vec3f64* v, double* out_ptr, uintptr_t out_len);

#ifdef __cplusplus
}
#endif

#endif // MOLCORE_H
