//! ForceField definition layer using ECS.
//!
//! This module implements atom and bond type definitions where:
//! - Each **AtomStyle** represents a class of atom representation
//! - Each **AtomType** belongs to one AtomStyle
//! - Each **BondStyle** represents a class of bond representation
//! - Each **BondType** belongs to one BondStyle and connects two AtomTypes
//! - Physical properties (charge, mass, spring constant, etc.) are individual components
//! - **ForceField** wraps the World and provides typed query APIs

pub mod components;
pub mod forcefield;

pub use components::{
    Alias, AtomStyle, AtomType, Charge, Mass, Symbol,
    BondStyle, BondType, SpringConstant, EquilibriumLength,
};
pub use forcefield::ForceField;
