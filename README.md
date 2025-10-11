# molcore

[![Crates.io](https://img.shields.io/crates/v/molcore.svg)](https://crates.io/crates/molcore)
[![Documentation](https://docs.rs/molcore/badge.svg)](https://docs.rs/molcore)
[![License](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)](LICENSE)

## Quick Start

### Testing

```bash
# Run fast unit tests
cargo test

# Run comprehensive integration tests (requires test data)
./scripts/fetch_testcases.sh
cargo test --features slow-tests
```

For more details, see [TESTING.md](TESTING.md).

## todo list
- [ ] Frame and Block: static data structures for aligned array
- [ ] ForceField: parameter storage and retrieval
- [ ] Array: typed n-dimensional array with axis labels
- [ ] Box: Simulation box with periodic boundary conditions
- [ ] IO: readers and writers for common file formats (XYZ frame and trajectory)
- [ ] c-api: C bindings for core functionality
- [ ] Python bindings via c-api

## Python bindings (experimental)

This repo includes a minimal Python package skeleton under `python/` to load the C FFI and provide zero-copy helpers for `Vec3` using NumPy buffers.

- Generate ctypes declarations from the C header:

```bash
python3 python/scripts/generate-declarations.py
```

- Build the C FFI shared library:

```bash
cargo build -p molcore-c --release
```

- Try the Vec3 helpers in Python:

```python
import numpy as np
from python.molcore_c import Vec3f32, Vec3f64

arr = np.array([1.0, 2.0, 3.0], dtype=np.float32)
v = Vec3f32.from_numpy(arr)  # passes pointer to Rust C-API without copying
out = v.to_numpy()           # returns a new numpy array with values
print(out)                   # [1. 2. 3.]
```

Notes
- The Python wrapper uses `ctypes` and expects the built library in `target/{debug,release}`.
- The conversion functions pass raw pointers from NumPy to the C API, avoiding copies when the array is C-contiguous with matching dtype.