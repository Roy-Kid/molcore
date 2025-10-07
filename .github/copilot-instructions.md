# Copilot Instructions for molcore

## Project Overview
molcore is a Rust molecular modeling library with a workspace architecture containing two main crates:
- **kernel** (`molcore-kernel`): Core library with element data, array abstractions, and a custom ECS
- **ffi** (`molcore-ffi`): `no_std` compatible C FFI layer (`cdylib` + `rlib`) with `#[no_mangle]` exports

This is a backend for molpy (Python molecular modeling toolkit). The FFI layer enables interop with Python/C.

## Architecture Patterns

### Workspace Structure
- Root `Cargo.toml` defines workspace with shared metadata (version, authors, license, etc.)
- `kernel/` builds as `molcore` lib (note: lib name differs from package name in `Cargo.toml`)
- `ffi/` depends on `kernel` via path dependency, exports C ABI
- Tests live in `kernel/tests/` using integration test pattern with nested modules

### Core Modules (`kernel/src/core/`)
1. **element.rs**: Static periodic table data via `ELEMENTS: &[Element]`. Lookup by atomic number or symbol (case-insensitive). Currently incomplete (only H defined).
2. **array/**: NumPy-inspired abstractions
   - `DType` enum for type metadata (Int8, Float32, etc.) with `HasDType` trait for Rust types
   - `Array` trait requires `dtype()` and `shape()` methods, plus `Send + Sync`
   - `Vec3<T>` generic 3D vector implementing `Array`
3. **ecs/**: Custom Entity Component System (not using existing ECS crates)
   - `Entity` is a lightweight u32 handle
   - `World` manages entities, components (via `HashMap<TypeId, Box<dyn Storage>>`), and resources
   - Components stored in sparse sets (`SparseSet<T>`) for cache-friendly iteration
   - `Schedule` runs systems sequentially (no parallelism yet)
   - Systems are `Fn(&mut World)` closures, registered via `Schedule::add_system()`

### FFI Layer Conventions
- All exports use `#[no_mangle]` and `extern "C"`
- Prefix all symbols with `mc_` (e.g., `mc_version`, `mc_status_error`)
- Status codes: `mc_status_t = i32`, where `0 = success`, non-zero = error
- Header in `ffi/include/molcore.h` manually maintained (no cbindgen yet)
- FFI structs mirror kernel types (e.g., `mc_vec3f32` for `Vec3<f32>`)

## Development Workflows

### Building & Testing
```bash
# Build all workspace members
cargo build

# Run tests (integration tests in kernel/tests/)
cargo test

# Test specific module
cargo test test_ecs

# Build FFI separately
cd ffi && cargo build --release
```

### Test Organization
- Integration tests in `kernel/tests/test_core.rs` declare nested `mod` hierarchy
- Each submodule maps to `test_core/<name>.rs` or `test_core/<name>/mod.rs`
- Example: `test_ecs/test_world.rs` contains ECS world tests
- Use simple test components like `struct Pos(f32)` deriving `Debug, Clone, Copy, PartialEq`

### Adding New Elements
Edit `kernel/src/core/element.rs` and append to `ELEMENTS` array. Follow existing pattern:
```rust
Element { z: 2, symbol: "He", name: "Helium", atomic_mass: 4.003 }
```

### Implementing Array Types
1. Define struct (e.g., `struct MyArray<T> { ... }`)
2. Impl `Array` trait with `dtype()` returning element `DType` and `shape()` returning dimensions
3. Ensure `Send + Sync` bounds (required by ECS components)

### Adding ECS Components
1. Define struct (any type that is `'static + Send + Sync`)
2. Insert via `world.insert_component(entity, component)`
3. Query via `world.query::<T>()` or `world.query_mut::<T>()`
4. For multi-component queries, use `world.query2::<A, B>()` (returns iterator of `(Entity, (&A, &B))`)

### Extending FFI
1. Add `#[no_mangle] pub extern "C"` function in `ffi/src/`
2. Use `mc_` prefix and `mc_status_t` return type
3. Update `ffi/include/molcore.h` manually
4. Handle null pointers safely (check before dereferencing)

## Project-Specific Conventions

### Type Bounds
- Array types: `Send + Sync` (enables use as ECS components)
- ECS components: `T: 'static + Send + Sync`
- System functions: `Fn(&mut World) + Send + Sync + 'static`

### Documentation
- All public items require doc comments (`#![warn(missing_docs)]` in `lib.rs`)
- Include examples in doc comments (tested via `cargo test`)
- Element lookup methods document panic conditions explicitly

### Error Handling
- Core library: Use `expect()` with descriptive messages (e.g., "invalid atomic number")
- FFI layer: Return `mc_status_t` codes, never panic across FFI boundary

### Naming
- Kernel modules: lowercase snake_case (e.g., `element`, `array`, `ecs`)
- FFI symbols: `mc_<module>_<action>` (e.g., `mc_vec3f32_from_ptr`)
- Test modules: `test_<name>` prefix (e.g., `test_world`, `test_dtype`)

## Key Files Reference
- `kernel/src/lib.rs`: Crate root with module declarations and feature flags
- `kernel/src/core/mod.rs`: Core module re-exports
- `kernel/src/core/ecs/world.rs`: Central ECS API (start here for ECS work)
- `ffi/src/lib.rs`: FFI crate root (`#![no_std]`)
- `ffi/include/molcore.h`: C API header (manually sync with Rust exports)

## Current State & Known Limitations
- Only Hydrogen implemented in periodic table
- No bond information or molecular structure types yet
- ECS scheduler is sequential (no parallel execution)
- FFI layer minimal (concept validation stage)
- No cbindgen automation (header manually maintained)
