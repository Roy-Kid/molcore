//! # molomni
//!
//! A Rust library providing core molecular modeling functionality.
//!
//! ## Features
//!
//! - Element data and lookup by atomic number or symbol
//! - Zero dependencies
//! - Type-safe molecular representations
//!
//! ## Examples
//!
//! ```
//! use molomni::core::Element;
//!
//! // Look up elements by atomic number
//! let hydrogen = Element::by_number(1);
//! assert_eq!(hydrogen.symbol, "H");
//!
//! // Or by symbol (case-insensitive)
//! let h = Element::by_symbol("h");
//! assert_eq!(h.name, "Hydrogen");
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod core;
