import ctypes
from typing import Tuple

import numpy as np

from ._lib import lib
from ._c_api import mc_vec3f32, mc_vec3f64


class Vec3f32:
    """Thin wrapper around mc_vec3f32 with zero-copy conversion to/from numpy.

    - from_numpy: consumes any array-like, uses np.asarray(dtype=float32), then
      passes the data pointer to C without copying (if already contiguous float32).
    - to_numpy: writes into a provided numpy array or returns a new one.
    """

    def __init__(self, x: float, y: float, z: float):
        self._c = mc_vec3f32(x=float(x), y=float(y), z=float(z))

    @classmethod
    def from_numpy(cls, arr: np.ndarray) -> "Vec3f32":
        a = np.asarray(arr, dtype=np.float32)
        if a.size < 3:
            raise ValueError("expected at least 3 elements")
        a_c = np.ascontiguousarray(a)
        ptr = a_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        out = mc_vec3f32()
        status = lib.mc_vec3f32_from_ptr(ptr, ctypes.byref(out))
        if status != 0:
            raise RuntimeError("mc_vec3f32_from_ptr failed")
        v = cls(0.0, 0.0, 0.0)
        v._c = out
        return v

    def to_numpy(self, out: np.ndarray | None = None) -> np.ndarray:
        if out is None:
            out = np.empty(3, dtype=np.float32)
        else:
            if out.dtype != np.float32 or out.size < 3:
                raise ValueError("out must be float32 with size >= 3")
            if not out.flags.c_contiguous:
                raise ValueError("out must be C contiguous")
        ptr = out.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        status = lib.mc_vec3f32_to_ptr(ctypes.byref(self._c), ptr, ctypes.c_size_t(out.size))
        if status != 0:
            raise RuntimeError("mc_vec3f32_to_ptr failed")
        return out

    @property
    def xyz(self) -> Tuple[float, float, float]:
        return float(self._c.x), float(self._c.y), float(self._c.z)

    @staticmethod
    def scale_inplace(arr: np.ndarray, alpha: float) -> None:
        a = np.asarray(arr, dtype=np.float32)
        if a.size < 3:
            raise ValueError("expected at least 3 elements")
        if not a.flags.c_contiguous:
            raise ValueError("array must be C contiguous")
        ptr = a.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        status = lib.mc_vec3f32_scale_inplace(ptr, ctypes.c_float(alpha), ctypes.c_size_t(a.size))
        if status != 0:
            raise RuntimeError("mc_vec3f32_scale_inplace failed")


class Vec3f64:
    def __init__(self, x: float, y: float, z: float):
        self._c = mc_vec3f64(x=float(x), y=float(y), z=float(z))

    @classmethod
    def from_numpy(cls, arr: np.ndarray) -> "Vec3f64":
        a = np.asarray(arr, dtype=np.float64)
        if a.size < 3:
            raise ValueError("expected at least 3 elements")
        a_c = np.ascontiguousarray(a)
        ptr = a_c.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        out = mc_vec3f64()
        status = lib.mc_vec3f64_from_ptr(ptr, ctypes.byref(out))
        if status != 0:
            raise RuntimeError("mc_vec3f64_from_ptr failed")
        v = cls(0.0, 0.0, 0.0)
        v._c = out
        return v

    def to_numpy(self, out: np.ndarray | None = None) -> np.ndarray:
        if out is None:
            out = np.empty(3, dtype=np.float64)
        else:
            if out.dtype != np.float64 or out.size < 3:
                raise ValueError("out must be float64 with size >= 3")
            if not out.flags.c_contiguous:
                raise ValueError("out must be C contiguous")
        ptr = out.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        status = lib.mc_vec3f64_to_ptr(ctypes.byref(self._c), ptr, ctypes.c_size_t(out.size))
        if status != 0:
            raise RuntimeError("mc_vec3f64_to_ptr failed")
        return out

    @property
    def xyz(self) -> Tuple[float, float, float]:
        return float(self._c.x), float(self._c.y), float(self._c.z)

    @staticmethod
    def add_inplace(a: np.ndarray, b: np.ndarray) -> None:
        aa = np.asarray(a, dtype=np.float32)
        bb = np.asarray(b, dtype=np.float32)
        if aa.size < 3 or bb.size < 3:
            raise ValueError("expected at least 3 elements")
        if not aa.flags.c_contiguous or not bb.flags.c_contiguous:
            raise ValueError("arrays must be C contiguous")
        ap = aa.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        bp = bb.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        status = lib.mc_vec3f32_add_inplace(ap, bp, ctypes.c_size_t(min(aa.size, bb.size)))
        if status != 0:
            raise RuntimeError("mc_vec3f32_add_inplace failed")

    @staticmethod
    def scale_inplace(arr: np.ndarray, alpha: float) -> None:
        a = np.asarray(arr, dtype=np.float64)
        if a.size < 3:
            raise ValueError("expected at least 3 elements")
        if not a.flags.c_contiguous:
            raise ValueError("array must be C contiguous")
        ptr = a.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        status = lib.mc_vec3f64_scale_inplace(ptr, ctypes.c_double(alpha), ctypes.c_size_t(a.size))
        if status != 0:
            raise RuntimeError("mc_vec3f64_scale_inplace failed")

    @staticmethod
    def add_inplace(a: np.ndarray, b: np.ndarray) -> None:
        aa = np.asarray(a, dtype=np.float64)
        bb = np.asarray(b, dtype=np.float64)
        if aa.size < 3 or bb.size < 3:
            raise ValueError("expected at least 3 elements")
        if not aa.flags.c_contiguous or not bb.flags.c_contiguous:
            raise ValueError("arrays must be C contiguous")
        ap = aa.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        bp = bb.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        status = lib.mc_vec3f64_add_inplace(ap, bp, ctypes.c_size_t(min(aa.size, bb.size)))
        if status != 0:
            raise RuntimeError("mc_vec3f64_add_inplace failed")
