//! Component definitions for forcefield atoms and properties.

use crate::core::ecs::Entity;

/// Represents a class of atom representation (e.g., "full", "united-atom", "coarse-grained").
///
/// An AtomStyle is an entity that groups related AtomTypes together.
///
/// # Examples
///
/// ```
/// use molomni::core::ecs::World;
/// use molomni::core::forcefield::AtomStyle;
///
/// let mut world = World::new();
/// let style = world.spawn();
/// world.insert_component(style, AtomStyle { name: "full".into() });
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomStyle {
    /// Name of the atom style (e.g., "full", "charge", "molecular")
    pub name: String,
}

/// Represents a specific atom type definition within a forcefield.
///
/// Each AtomType belongs to exactly one AtomStyle and can have various
/// property components attached (Symbol, Mass, Charge, etc.).
///
/// # Examples
///
/// ```
/// use molomni::core::ecs::World;
/// use molomni::core::forcefield::{AtomStyle, AtomType, Symbol, Mass};
///
/// let mut world = World::new();
/// let style = world.spawn();
/// world.insert_component(style, AtomStyle { name: "full".into() });
///
/// let atom_type = world.spawn();
/// world.insert_component(atom_type, AtomType { 
///     name: "C_sp2".into(), 
///     style 
/// });
/// world.insert_component(atom_type, Symbol("C".into()));
/// world.insert_component(atom_type, Mass(12.011));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomType {
    /// Name of the atom type (e.g., "C_sp2", "Li+", "water_oxygen")
    pub name: String,
    /// Entity reference to the parent AtomStyle
    pub style: Entity,
}

/// Chemical symbol for an atom type.
///
/// # Examples
///
/// ```
/// use molomni::core::forcefield::Symbol;
///
/// let carbon = Symbol("C".into());
/// let lithium = Symbol("Li".into());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol(pub String);

/// Human-readable alias or description for an atom type.
///
/// # Examples
///
/// ```
/// use molomni::core::forcefield::Alias;
///
/// let alias = Alias("Lithium ion".into());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias(pub String);

/// Partial charge on an atom type in elementary charge units (e).
///
/// # Examples
///
/// ```
/// use molomni::core::forcefield::Charge;
///
/// let cation = Charge(1.0);
/// let anion = Charge(-1.0);
/// let neutral = Charge(0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Charge(pub f64);

/// Atomic mass in unified atomic mass units (u or Da).
///
/// # Examples
///
/// ```
/// use molomni::core::forcefield::Mass;
///
/// let carbon = Mass(12.011);
/// let hydrogen = Mass(1.008);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mass(pub f64);

/// Represents a class of bond representation (e.g., "harmonic", "morse", "FENE").
///
/// A BondStyle is an entity that groups related BondTypes together.
///
/// # Examples
///
/// ```
/// use molomni::core::ecs::World;
/// use molomni::core::forcefield::BondStyle;
///
/// let mut world = World::new();
/// let style = world.spawn();
/// world.insert_component(style, BondStyle { name: "harmonic".into() });
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BondStyle {
    /// Name of the bond style (e.g., "harmonic", "morse", "FENE")
    pub name: String,
}

/// Represents a specific bond type definition within a forcefield.
///
/// Each BondType belongs to exactly one BondStyle and connects two AtomTypes.
/// BondTypes can have property components (e.g., spring constant, equilibrium length).
///
/// **Important**: The two `AtomType` entities must exist before creating a `BondType`.
///
/// # Examples
///
/// ```
/// use molomni::core::forcefield::{ForceField, BondStyle, BondType};
///
/// let mut ff = ForceField::new();
/// let atom_style = ff.create_style("full");
/// let bond_style = ff.create_bond_style("harmonic");
///
/// // Create atom types first
/// let c_type = ff.create_type("C", atom_style);
/// let h_type = ff.create_type("H", atom_style);
///
/// // Now create bond type between them
/// let ch_bond = ff.create_bond_type("C-H", bond_style, c_type, h_type);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BondType {
    /// Name of the bond type (e.g., "C-H", "C=C", "peptide")
    pub name: String,
    /// Entity reference to the parent BondStyle
    pub style: Entity,
    /// First atom type in the bond
    pub atom1: Entity,
    /// Second atom type in the bond
    pub atom2: Entity,
}

/// Harmonic bond spring constant (force constant) in energy/distance² units.
///
/// Commonly used in harmonic potential: E = k/2 * (r - r0)²
///
/// # Examples
///
/// ```
/// use molomni::core::forcefield::SpringConstant;
///
/// let k = SpringConstant(340.0); // kcal/mol/Å²
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConstant(pub f64);

/// Equilibrium bond length in Angstroms.
///
/// # Examples
///
/// ```
/// use molomni::core::forcefield::EquilibriumLength;
///
/// let r0 = EquilibriumLength(1.09); // Å
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquilibriumLength(pub f64);
