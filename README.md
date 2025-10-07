# molcore

[![Crates.io](https://img.shields.io/crates/v/molcore.svg)](https://crates.io/crates/molcore)
[![Documentation](https://docs.rs/molcore/badge.svg)](https://docs.rs/molcore)
[![License](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)](LICENSE)

A Rust library providing core molecular modeling functionality, including element data and molecular representations.

**Note**: This is a work-in-progress backend for molpy.

## Features

- 🧪 **Element Data**: Complete periodic table information
  - Access elements by atomic number or symbol
  - Case-insensitive symbol lookup
  - Atomic mass, names, and properties
  
- 🚀 **Zero Dependencies**: Lightweight and fast
- 🔒 **Type Safe**: Leverages Rust's type system for safety
- 📦 **`no_std` Compatible**: Can be used in embedded environments

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
molcore = "0.1"
```

## Usage

```rust
use molcore::core::Element;

fn main() {
    // Look up elements by atomic number
    let hydrogen = Element::by_number(1);
    println!("{}: {}", hydrogen.symbol, hydrogen.name);
    // Output: H: Hydrogen
    
    // Look up elements by symbol (case-insensitive)
    let h1 = Element::by_symbol("H");
    let h2 = Element::by_symbol("h");
    assert_eq!(h1 as *const _, h2 as *const _); // Same reference
    
    // Access element properties
    println!("Atomic mass: {}", hydrogen.atomic_mass);
    println!("Atomic number: {}", hydrogen.z);
}
```

## API Overview

### `Element` Struct

```rust
pub struct Element {
    pub z: u8,                    // Atomic number
    pub symbol: &'static str,     // Element symbol
    pub name: &'static str,       // Element name
    pub atomic_mass: f32,         // Atomic mass in u
}
```

### Methods

- `Element::by_number(z: u8) -> &'static Element` - Find element by atomic number
- `Element::by_symbol(sym: &str) -> &'static Element` - Find element by symbol (case-insensitive)

### Panics

Both lookup methods will panic with descriptive messages if the element is not found:
- `by_number`: panics with "invalid atomic number"
- `by_symbol`: panics with "invalid symbol"

## Development

### Running Tests

```bash
cargo test
```

### Building Documentation

```bash
cargo doc --open
```

## Roadmap

- [ ] Complete periodic table data (currently only H)
- [ ] Molecular structure representations
- [ ] Bond information
- [ ] Additional element properties (electronegativity, radius, etc.)
- [ ] Python bindings via PyO3

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the BSD 3-Clause License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Part of the MolCrafts project ecosystem.
