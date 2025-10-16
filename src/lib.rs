//! # molcore
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
//! use molcore::core::Element;
//!
//! // Look up elements by atomic number
//! let hydrogen = Element::by_number(1).unwrap();
//! assert_eq!(hydrogen.symbol(), "H");
//!
//! // Or by symbol (case-insensitive)
//! let h = Element::by_symbol("h").unwrap();
//! assert_eq!(h.name(), "Hydrogen");
//! ```

#![allow(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod core;
pub mod io;
pub mod math;