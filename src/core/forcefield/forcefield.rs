//! ForceField data structure for managing atom types, bond types, and properties.

use crate::core::ecs::{Entity, World};
use super::components::{AtomStyle, AtomType, BondStyle, BondType};

/// ForceField wraps an ECS World and provides typed APIs for managing
/// atom styles, atom types, and their properties.
///
/// # Generic Query Pattern:
/// Instead of hardcoded functions like `query_types_with_mass()`,
/// use generic `query_with_property::<T>()` for any property component type.
///
/// # Examples
///
/// ```
/// use molcore::core::forcefield::{ForceField, Symbol, Mass, Charge};
///
/// let mut ff = ForceField::new();
///
/// // Create a style
/// let style = ff.create_style("full");
///
/// // Create an atom type with properties
/// let carbon = ff.create_type("C_sp2", style);
/// ff.set_property(carbon, Symbol("C".into()));
/// ff.set_property(carbon, Mass(12.011));
///
/// // Query by property type - generic!
/// let with_mass = ff.query_with_property::<Mass>();
/// assert_eq!(with_mass.len(), 1);
///
/// // Query with multiple properties - also generic!
/// let with_both = ff.query_with_properties::<Symbol, Mass>();
/// assert_eq!(with_both.len(), 1);
/// ```
pub struct ForceField {
    world: World,
}

impl ForceField {
    /// Create a new empty ForceField.
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    /// Get immutable reference to the underlying World.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get mutable reference to the underlying World.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Create a new AtomStyle and return its entity.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    /// ```
    pub fn create_style(&mut self, name: impl Into<String>) -> Entity {
        let entity = self.world.spawn();
        self.world.insert_component(entity, AtomStyle { name: name.into() });
        entity
    }

    /// Create a new AtomType belonging to a style and return its entity.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    /// let atom_type = ff.create_type("C_sp2", style);
    /// ```
    pub fn create_type(&mut self, name: impl Into<String>, style: Entity) -> Entity {
        let entity = self.world.spawn();
        self.world.insert_component(entity, AtomType { 
            name: name.into(), 
            style 
        });
        entity
    }

    /// Set a property component on an atom type.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::{ForceField, Mass, Charge};
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    /// let atom = ff.create_type("Li+", style);
    ///
    /// ff.set_property(atom, Mass(6.94));
    /// ff.set_property(atom, Charge(1.0));
    /// ```
    pub fn set_property<T: 'static + Send + Sync>(&mut self, entity: Entity, property: T) {
        self.world.insert_component(entity, property);
    }

    /// Get a property component from an atom type.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::{ForceField, Mass};
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    /// let atom = ff.create_type("C", style);
    /// ff.set_property(atom, Mass(12.011));
    ///
    /// let mass = ff.get_property::<Mass>(atom);
    /// assert_eq!(mass.unwrap().0, 12.011);
    /// ```
    pub fn get_property<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.world.get_component::<T>(entity)
    }

    /// Get a mutable property component from an atom type.
    pub fn get_property_mut<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<&mut T> {
        self.world.get_component_mut::<T>(entity)
    }

    /// Query all atom types that have a specific property component.
    ///
    /// Returns a vector of (Entity, &AtomType, &T) tuples.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::{ForceField, Mass};
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    /// 
    /// let c = ff.create_type("C", style);
    /// ff.set_property(c, Mass(12.011));
    ///
    /// let h = ff.create_type("H", style);
    /// ff.set_property(h, Mass(1.008));
    ///
    /// let with_mass = ff.query_with_property::<Mass>();
    /// assert_eq!(with_mass.len(), 2);
    /// ```
    pub fn query_with_property<T: 'static>(&self) -> Vec<(Entity, &AtomType, &T)> {
        self.world.query2::<AtomType, T>()
            .map(|(e, (at, prop))| (e, at, prop))
            .collect()
    }

    /// Query all bond types that have a specific property component.
    ///
    /// Returns a vector of (Entity, &BondType, &T) tuples.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::{ForceField, SpringConstant};
    ///
    /// let mut ff = ForceField::new();
    /// let atom_style = ff.create_style("full");
    /// let bond_style = ff.create_bond_style("harmonic");
    /// 
    /// let c = ff.create_type("C", atom_style);
    /// let h = ff.create_type("H", atom_style);
    /// 
    /// let ch = ff.create_bond_type("C-H", bond_style, c, h);
    /// ff.set_property(ch, SpringConstant(340.0));
    ///
    /// let with_k = ff.query_bonds_with_property::<SpringConstant>();
    /// assert_eq!(with_k.len(), 1);
    /// ```
    pub fn query_bonds_with_property<T: 'static>(&self) -> Vec<(Entity, &BondType, &T)> {
        self.world.query2::<BondType, T>()
            .map(|(e, (bt, prop))| (e, bt, prop))
            .collect()
    }

    /// Query all atom types belonging to a specific style.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let style1 = ff.create_style("full");
    /// let style2 = ff.create_style("united");
    ///
    /// ff.create_type("C1", style1);
    /// ff.create_type("C2", style1);
    /// ff.create_type("CH3", style2);
    ///
    /// let types = ff.query_types_of_style(style1);
    /// assert_eq!(types.len(), 2);
    /// ```
    pub fn query_types_of_style(&self, style: Entity) -> Vec<(Entity, String)> {
        self.world.query::<AtomType>()
            .iter()
            .filter(|(_, at)| at.style == style)
            .map(|(e, at)| (e, at.name.clone()))
            .collect()
    }

    /// Find an atom type entity by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    /// let carbon = ff.create_type("C_sp2", style);
    ///
    /// let found = ff.find_type_by_name("C_sp2");
    /// assert_eq!(found, Some(carbon));
    /// ```
    pub fn find_type_by_name(&self, name: &str) -> Option<Entity> {
        self.world.query::<AtomType>()
            .iter()
            .find(|(_, at)| at.name == name)
            .map(|(e, _)| e)
    }

    /// Get all atom styles.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// ff.create_style("full");
    /// ff.create_style("united");
    ///
    /// let styles = ff.get_styles();
    /// assert_eq!(styles.len(), 2);
    /// ```
    pub fn get_styles(&self) -> Vec<(Entity, String)> {
        self.world.query::<AtomStyle>()
            .iter()
            .map(|(e, s)| (e, s.name.clone()))
            .collect()
    }

    /// Find a style entity by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    ///
    /// let found = ff.find_style_by_name("full");
    /// assert_eq!(found, Some(style));
    /// ```
    pub fn find_style_by_name(&self, name: &str) -> Option<Entity> {
        self.world.query::<AtomStyle>()
            .iter()
            .find(|(_, s)| s.name == name)
            .map(|(e, _)| e)
    }

    /// Get all atom types.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    /// ff.create_type("C", style);
    /// ff.create_type("H", style);
    ///
    /// let types = ff.get_types();
    /// assert_eq!(types.len(), 2);
    /// ```
    pub fn get_types(&self) -> Vec<(Entity, String)> {
        self.world.query::<AtomType>()
            .iter()
            .map(|(e, at)| (e, at.name.clone()))
            .collect()
    }

    // ============ Bond Management ============

    /// Create a new BondStyle and return its entity.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let bond_style = ff.create_bond_style("harmonic");
    /// ```
    pub fn create_bond_style(&mut self, name: impl Into<String>) -> Entity {
        let entity = self.world.spawn();
        self.world.insert_component(entity, BondStyle { name: name.into() });
        entity
    }

    /// Create a new BondType connecting two AtomTypes.
    ///
    /// **Important**: Both `atom1` and `atom2` must be valid AtomType entities.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let atom_style = ff.create_style("full");
    /// let bond_style = ff.create_bond_style("harmonic");
    ///
    /// let c = ff.create_type("C", atom_style);
    /// let h = ff.create_type("H", atom_style);
    ///
    /// let ch_bond = ff.create_bond_type("C-H", bond_style, c, h);
    /// ```
    pub fn create_bond_type(
        &mut self,
        name: impl Into<String>,
        style: Entity,
        atom1: Entity,
        atom2: Entity,
    ) -> Entity {
        let entity = self.world.spawn();
        self.world.insert_component(entity, BondType {
            name: name.into(),
            style,
            atom1,
            atom2,
        });
        entity
    }

    /// Query all bond types belonging to a specific bond style.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let atom_style = ff.create_style("full");
    /// let harmonic = ff.create_bond_style("harmonic");
    /// let morse = ff.create_bond_style("morse");
    ///
    /// let c = ff.create_type("C", atom_style);
    /// let h = ff.create_type("H", atom_style);
    ///
    /// ff.create_bond_type("C-H_1", harmonic, c, h);
    /// ff.create_bond_type("C-H_2", harmonic, c, h);
    /// ff.create_bond_type("C-H_morse", morse, c, h);
    ///
    /// let harmonic_bonds = ff.query_bond_types_of_style(harmonic);
    /// assert_eq!(harmonic_bonds.len(), 2);
    /// ```
    pub fn query_bond_types_of_style(&self, style: Entity) -> Vec<(Entity, String)> {
        self.world.query::<BondType>()
            .iter()
            .filter(|(_, bt)| bt.style == style)
            .map(|(e, bt)| (e, bt.name.clone()))
            .collect()
    }

    /// Find a bond type by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let atom_style = ff.create_style("full");
    /// let bond_style = ff.create_bond_style("harmonic");
    ///
    /// let c = ff.create_type("C", atom_style);
    /// let h = ff.create_type("H", atom_style);
    /// let ch_bond = ff.create_bond_type("C-H", bond_style, c, h);
    ///
    /// let found = ff.find_bond_type_by_name("C-H");
    /// assert_eq!(found, Some(ch_bond));
    /// ```
    pub fn find_bond_type_by_name(&self, name: &str) -> Option<Entity> {
        self.world.query::<BondType>()
            .iter()
            .find(|(_, bt)| bt.name == name)
            .map(|(e, _)| e)
    }

    /// Get all bond styles.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// ff.create_bond_style("harmonic");
    /// ff.create_bond_style("morse");
    ///
    /// let styles = ff.get_bond_styles();
    /// assert_eq!(styles.len(), 2);
    /// ```
    pub fn get_bond_styles(&self) -> Vec<(Entity, String)> {
        self.world.query::<BondStyle>()
            .iter()
            .map(|(e, s)| (e, s.name.clone()))
            .collect()
    }

    /// Find a bond style by name.
    pub fn find_bond_style_by_name(&self, name: &str) -> Option<Entity> {
        self.world.query::<BondStyle>()
            .iter()
            .find(|(_, s)| s.name == name)
            .map(|(e, _)| e)
    }

    /// Get all bond types.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let atom_style = ff.create_style("full");
    /// let bond_style = ff.create_bond_style("harmonic");
    ///
    /// let c = ff.create_type("C", atom_style);
    /// let h = ff.create_type("H", atom_style);
    ///
    /// ff.create_bond_type("C-H", bond_style, c, h);
    /// ff.create_bond_type("C-C", bond_style, c, c);
    ///
    /// let bonds = ff.get_bond_types();
    /// assert_eq!(bonds.len(), 2);
    /// ```
    pub fn get_bond_types(&self) -> Vec<(Entity, String)> {
        self.world.query::<BondType>()
            .iter()
            .map(|(e, bt)| (e, bt.name.clone()))
            .collect()
    }

    /// Query all bonds connecting a specific atom type.
    ///
    /// Returns bonds where the atom appears as either atom1 or atom2.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let atom_style = ff.create_style("full");
    /// let bond_style = ff.create_bond_style("harmonic");
    ///
    /// let c = ff.create_type("C", atom_style);
    /// let h = ff.create_type("H", atom_style);
    /// let o = ff.create_type("O", atom_style);
    ///
    /// ff.create_bond_type("C-H", bond_style, c, h);
    /// ff.create_bond_type("C-O", bond_style, c, o);
    /// ff.create_bond_type("H-O", bond_style, h, o);
    ///
    /// let c_bonds = ff.query_bonds_with_atom(c);
    /// assert_eq!(c_bonds.len(), 2); // C-H and C-O
    /// ```
    pub fn query_bonds_with_atom(&self, atom: Entity) -> Vec<(Entity, String)> {
        self.world.query::<BondType>()
            .iter()
            .filter(|(_, bt)| bt.atom1 == atom || bt.atom2 == atom)
            .map(|(e, bt)| (e, bt.name.clone()))
            .collect()
    }

    /// Query all bonds between two specific atom types.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::ForceField;
    ///
    /// let mut ff = ForceField::new();
    /// let atom_style = ff.create_style("full");
    /// let bond_style = ff.create_bond_style("harmonic");
    ///
    /// let c = ff.create_type("C", atom_style);
    /// let h = ff.create_type("H", atom_style);
    ///
    /// ff.create_bond_type("C-H_1", bond_style, c, h);
    /// ff.create_bond_type("C-H_2", bond_style, c, h);
    ///
    /// let ch_bonds = ff.query_bonds_between_atoms(c, h);
    /// assert_eq!(ch_bonds.len(), 2);
    /// ```
    pub fn query_bonds_between_atoms(&self, atom1: Entity, atom2: Entity) -> Vec<(Entity, String)> {
        self.world.query::<BondType>()
            .iter()
            .filter(|(_, bt)| {
                (bt.atom1 == atom1 && bt.atom2 == atom2) ||
                (bt.atom1 == atom2 && bt.atom2 == atom1)
            })
            .map(|(e, bt)| (e, bt.name.clone()))
            .collect()
    }

    /// Query atom types that have two specific properties.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::{ForceField, Symbol, Mass};
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    /// 
    /// let c = ff.create_type("C", style);
    /// ff.set_property(c, Symbol("C".into()));
    /// ff.set_property(c, Mass(12.011));
    ///
    /// let with_both = ff.query_with_properties::<Symbol, Mass>();
    /// assert_eq!(with_both.len(), 1);
    /// ```
    pub fn query_with_properties<A: 'static, B: 'static>(&self) -> Vec<(Entity, &AtomType, &A, &B)> {
        self.world.query2::<A, B>()
            .filter_map(|(e, (a, b))| {
                self.world.get_component::<AtomType>(e)
                    .map(|at| (e, at, a, b))
            })
            .collect()
    }

    /// Print a summary of the forcefield contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use molcore::core::forcefield::{ForceField, Mass};
    ///
    /// let mut ff = ForceField::new();
    /// let style = ff.create_style("full");
    /// let carbon = ff.create_type("C", style);
    /// ff.set_property(carbon, Mass(12.011));
    /// // Ensure bond-related storages are registered for summary printing
    /// let bond_style = ff.create_bond_style("harmonic");
    /// let hydrogen = ff.create_type("H", style);
    /// let _bond = ff.create_bond_type("C-H", bond_style, carbon, hydrogen);
    ///
    /// ff.print_summary();
    /// ```
    pub fn print_summary(&self) {
        println!("=== ForceField Summary ===\n");

        println!("Atom Styles:");
        for (entity, name) in self.get_styles() {
            println!("  {:?}: {}", entity, name);
        }
        println!();

        println!("Atom Types:");
        for (entity, name) in self.get_types() {
            if let Some(at) = self.world.get_component::<AtomType>(entity) {
                println!("  {:?}: {} (style: {:?})", entity, name, at.style);
            }
        }
        println!();

        println!("Bond Styles:");
        for (entity, name) in self.get_bond_styles() {
            println!("  {:?}: {}", entity, name);
        }
        println!();

        println!("Bond Types:");
        for (entity, name) in self.get_bond_types() {
            if let Some(bt) = self.world.get_component::<BondType>(entity) {
                println!("  {:?}: {} (style: {:?}, {} ↔ {})",
                    entity, name, bt.style, 
                    self.world.get_component::<AtomType>(bt.atom1)
                        .map(|at| at.name.as_str())
                        .unwrap_or("?"),
                    self.world.get_component::<AtomType>(bt.atom2)
                        .map(|at| at.name.as_str())
                        .unwrap_or("?")
                );
            }
        }
    }
}

impl Default for ForceField {
    fn default() -> Self {
        Self::new()
    }
}
