# fmt: off
# flake8: noqa

import ctypes
import enum
import platform
from ctypes import CFUNCTYPE, POINTER


arch = platform.architecture()[0]
if arch == "32bit":
    c_uintptr_t = ctypes.c_uint32
elif arch == "64bit":
    c_uintptr_t = ctypes.c_uint64



mc_status_t = ctypes.c_int32


class mc_array_t(ctypes.Structure):
    pass

mc_array_t._fields_ = [
    ("shape", POINTER(ctypes.c_int64)),
    ("shape_len", c_uintptr_t),
    ("dtype", mc_dtype_t),
]


class mc_vec3f32(ctypes.Structure):
    pass

mc_vec3f32._fields_ = [
    ("x", ctypes.c_float),
    ("y", ctypes.c_float),
    ("z", ctypes.c_float),
]


class mc_vec3f64(ctypes.Structure):
    pass

mc_vec3f64._fields_ = [
    ("x", ctypes.c_double),
    ("y", ctypes.c_double),
    ("z", ctypes.c_double),
]


def setup_functions(lib):
    return lib

    lib.mc_version.argtypes = [
    ]
    lib.mc_version.restype = ctypes.c_uint32

    lib.mc_status_error.argtypes = [
        mc_status_t,
    ]
    lib.mc_status_error.restype = mc_status_t

    lib.mc_status_from_bool.argtypes = [
        ctypes.c__Bool,
    ]
    lib.mc_status_from_bool.restype = mc_status_t

    lib.mc_status_message.argtypes = [
        mc_status_t,
    ]
    lib.mc_status_message.restype = ctypes.c_char_p

    lib.mc_array_null.argtypes = [
    ]
    lib.mc_array_null.restype = POINTER(mc_array_t)

    lib.mc_array_is_null.argtypes = [
        POINTER(mc_array_t),
    ]
    lib.mc_array_is_null.restype = ctypes.c__Bool

    lib.mc_vec3f32_from_ptr.argtypes = [
        POINTER(ctypes.c_float),
        POINTER(mc_vec3f32),
    ]
    lib.mc_vec3f32_from_ptr.restype = mc_status_t

    lib.mc_vec3f32_to_ptr.argtypes = [
        POINTER(mc_vec3f32),
        POINTER(ctypes.c_float),
        c_uintptr_t,
    ]
    lib.mc_vec3f32_to_ptr.restype = mc_status_t

    lib.mc_vec3f64_from_ptr.argtypes = [
        POINTER(ctypes.c_double),
        POINTER(mc_vec3f64),
    ]
    lib.mc_vec3f64_from_ptr.restype = mc_status_t

    lib.mc_vec3f64_to_ptr.argtypes = [
        POINTER(mc_vec3f64),
        POINTER(ctypes.c_double),
        c_uintptr_t,
    ]
    lib.mc_vec3f64_to_ptr.restype = mc_status_t

    lib.mc_vec3f32_scale_inplace.argtypes = [
        POINTER(ctypes.c_float),
        ctypes.c_float,
        c_uintptr_t,
    ]
    lib.mc_vec3f32_scale_inplace.restype = mc_status_t

    lib.mc_vec3f32_add_inplace.argtypes = [
        POINTER(ctypes.c_float),
        POINTER(ctypes.c_float),
        c_uintptr_t,
    ]
    lib.mc_vec3f32_add_inplace.restype = mc_status_t

    lib.mc_vec3f64_scale_inplace.argtypes = [
        POINTER(ctypes.c_double),
        ctypes.c_double,
        c_uintptr_t,
    ]
    lib.mc_vec3f64_scale_inplace.restype = mc_status_t

    lib.mc_vec3f64_add_inplace.argtypes = [
        POINTER(ctypes.c_double),
        POINTER(ctypes.c_double),
        c_uintptr_t,
    ]
    lib.mc_vec3f64_add_inplace.restype = mc_status_t
