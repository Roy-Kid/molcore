use molcore::core::forcefield::{ForceField, Symbol, Alias, Charge, Mass};

#[test]
fn create_atom_style() {
    let mut ff = ForceField::new();
    let _style = ff.create_style("full");

    let styles = ff.get_styles();
    assert_eq!(styles.len(), 1);
    assert_eq!(styles[0].1, "full");
}

#[test]
fn create_atom_type_with_style() {
    let mut ff = ForceField::new();
    
    // Create style
    let style = ff.create_style("full");
    
    // Create atom type
    let atom_type = ff.create_type("C_sp2", style);

    let found = ff.find_type_by_name("C_sp2");
    assert_eq!(found, Some(atom_type));
}

#[test]
fn attach_multiple_properties() {
    let mut ff = ForceField::new();
    
    let style = ff.create_style("full");
    let atom = ff.create_type("Li+", style);
    
    ff.set_property(atom, Symbol("Li".into()));
    ff.set_property(atom, Alias("Lithium ion".into()));
    ff.set_property(atom, Charge(1.0));
    ff.set_property(atom, Mass(6.94));

    assert_eq!(ff.get_property::<Symbol>(atom).unwrap().0, "Li");
    assert_eq!(ff.get_property::<Alias>(atom).unwrap().0, "Lithium ion");
    assert_eq!(ff.get_property::<Charge>(atom).unwrap().0, 1.0);
    assert_eq!(ff.get_property::<Mass>(atom).unwrap().0, 6.94);
}

#[test]
fn query_types_by_style() {
    let mut ff = ForceField::new();
    
    let style1 = ff.create_style("full");
    let style2 = ff.create_style("united");
    
    // Create atoms for style1
    ff.create_type("C1", style1);
    ff.create_type("C2", style1);
    
    // Create atom for style2
    ff.create_type("CH3", style2);

    // Query atoms of style1
    let style1_types = ff.query_types_of_style(style1);
    
    assert_eq!(style1_types.len(), 2);
}

#[test]
fn query_types_with_property_generic() {
    let mut ff = ForceField::new();
    
    let style = ff.create_style("full");
    
    // Atom with charge
    let ion = ff.create_type("Li+", style);
    ff.set_property(ion, Charge(1.0));
    
    // Atom without charge
    let _neutral = ff.create_type("C", style);

    // Query only atoms with charge - GENERIC!
    let charged = ff.query_with_property::<Charge>();
    
    assert_eq!(charged.len(), 1);
    assert_eq!(charged[0].1.name, "Li+");
}

#[test]
fn optional_properties() {
    let mut ff = ForceField::new();
    
    let style = ff.create_style("full");
    
    // Atom with minimal properties
    let minimal = ff.create_type("X", style);
    ff.set_property(minimal, Symbol("X".into()));
    
    // Atom with all properties
    let complete = ff.create_type("O", style);
    ff.set_property(complete, Symbol("O".into()));
    ff.set_property(complete, Mass(15.999));
    ff.set_property(complete, Charge(-0.8));
    ff.set_property(complete, Alias("Water oxygen".into()));

    // Verify minimal has only what was set
    assert!(ff.get_property::<Symbol>(minimal).is_some());
    assert!(ff.get_property::<Mass>(minimal).is_none());
    assert!(ff.get_property::<Charge>(minimal).is_none());
    assert!(ff.get_property::<Alias>(minimal).is_none());

    // Verify complete has everything
    assert!(ff.get_property::<Symbol>(complete).is_some());
    assert!(ff.get_property::<Mass>(complete).is_some());
    assert!(ff.get_property::<Charge>(complete).is_some());
    assert!(ff.get_property::<Alias>(complete).is_some());
}

#[test]
fn multiple_styles_independent() {
    let mut ff = ForceField::new();
    
    let full_style = ff.create_style("full");
    let ua_style = ff.create_style("united-atom");
    let cg_style = ff.create_style("coarse-grained");

    // Create atoms for each style
    ff.create_type("C_full", full_style);
    ff.create_type("CH3", ua_style);
    ff.create_type("BB", cg_style);

    // Verify each style has exactly one atom
    for style in [full_style, ua_style, cg_style] {
        let types = ff.query_types_of_style(style);
        assert_eq!(types.len(), 1);
    }
}

#[test]
fn query_with_multiple_properties_generic() {
    let mut ff = ForceField::new();
    
    let style = ff.create_style("full");
    
    let c = ff.create_type("C", style);
    ff.set_property(c, Symbol("C".into()));
    ff.set_property(c, Mass(12.011));
    
    let h = ff.create_type("H", style);
    ff.set_property(h, Symbol("H".into()));
    // H has no mass
    
    let li = ff.create_type("Li+", style);
    ff.set_property(li, Mass(6.94));
    // Li has no symbol

    // Generic query for types with BOTH Symbol AND Mass
    let with_both = ff.query_with_properties::<Symbol, Mass>();
    assert_eq!(with_both.len(), 1);
    assert_eq!(with_both[0].1.name, "C");
}

#[test]
fn query_different_property_types() {
    let mut ff = ForceField::new();
    
    let style = ff.create_style("full");
    
    // Create various atoms with different properties
    let c = ff.create_type("C", style);
    ff.set_property(c, Mass(12.011));
    
    let li = ff.create_type("Li+", style);
    ff.set_property(li, Charge(1.0));
    
    let o = ff.create_type("O", style);
    ff.set_property(o, Alias("Oxygen".into()));

    // Query each property type generically
    let with_mass = ff.query_with_property::<Mass>();
    let with_charge = ff.query_with_property::<Charge>();
    let with_alias = ff.query_with_property::<Alias>();

    assert_eq!(with_mass.len(), 1);
    assert_eq!(with_charge.len(), 1);
    assert_eq!(with_alias.len(), 1);
}

// ============ Bond Tests ============

#[test]
fn create_bond_style() {
    let mut ff = ForceField::new();
    let _bond_style = ff.create_bond_style("harmonic");

    let styles = ff.get_bond_styles();
    assert_eq!(styles.len(), 1);
    assert_eq!(styles[0].1, "harmonic");
}

#[test]
fn create_bond_type_between_atoms() {
    use molcore::core::forcefield::BondType;
    
    let mut ff = ForceField::new();
    
    // First create atom types
    let atom_style = ff.create_style("full");
    let c = ff.create_type("C", atom_style);
    let h = ff.create_type("H", atom_style);
    
    // Then create bond style and type
    let bond_style = ff.create_bond_style("harmonic");
    let ch_bond = ff.create_bond_type("C-H", bond_style, c, h);
    
    // Verify bond was created
    let found = ff.find_bond_type_by_name("C-H");
    assert_eq!(found, Some(ch_bond));
    
    // Verify bond references correct atoms
    let bond_data = ff.world().get_component::<BondType>(ch_bond).unwrap();
    assert_eq!(bond_data.atom1, c);
    assert_eq!(bond_data.atom2, h);
}

#[test]
fn query_bonds_with_atom() {
    let mut ff = ForceField::new();
    
    let atom_style = ff.create_style("full");
    let c = ff.create_type("C", atom_style);
    let h = ff.create_type("H", atom_style);
    let o = ff.create_type("O", atom_style);
    
    let bond_style = ff.create_bond_style("harmonic");
    ff.create_bond_type("C-H", bond_style, c, h);
    ff.create_bond_type("C-O", bond_style, c, o);
    ff.create_bond_type("H-O", bond_style, h, o);
    
    // Query bonds involving C
    let c_bonds = ff.query_bonds_with_atom(c);
    assert_eq!(c_bonds.len(), 2);
    
    // Query bonds involving O
    let o_bonds = ff.query_bonds_with_atom(o);
    assert_eq!(o_bonds.len(), 2);
}

#[test]
fn query_bonds_between_specific_atoms() {
    let mut ff = ForceField::new();
    
    let atom_style = ff.create_style("full");
    let c = ff.create_type("C", atom_style);
    let h = ff.create_type("H", atom_style);
    
    let bond_style = ff.create_bond_style("harmonic");
    ff.create_bond_type("C-H_1", bond_style, c, h);
    ff.create_bond_type("C-H_2", bond_style, c, h);
    ff.create_bond_type("C-C", bond_style, c, c);
    
    // Query C-H bonds
    let ch_bonds = ff.query_bonds_between_atoms(c, h);
    assert_eq!(ch_bonds.len(), 2);
    
    // Query C-C bonds
    let cc_bonds = ff.query_bonds_between_atoms(c, c);
    assert_eq!(cc_bonds.len(), 1);
}

#[test]
fn bond_properties_generic() {
    use molcore::core::forcefield::{SpringConstant, EquilibriumLength};
    
    let mut ff = ForceField::new();
    
    let atom_style = ff.create_style("full");
    let c = ff.create_type("C", atom_style);
    let h = ff.create_type("H", atom_style);
    
    let bond_style = ff.create_bond_style("harmonic");
    let ch_bond = ff.create_bond_type("C-H", bond_style, c, h);
    
    // Set bond properties
    ff.set_property(ch_bond, SpringConstant(340.0));
    ff.set_property(ch_bond, EquilibriumLength(1.09));
    
    // Query bonds with specific properties - GENERIC!
    let with_k = ff.query_bonds_with_property::<SpringConstant>();
    let with_r0 = ff.query_bonds_with_property::<EquilibriumLength>();
    
    assert_eq!(with_k.len(), 1);
    assert_eq!(with_r0.len(), 1);
    
    // Verify values
    assert_eq!(ff.get_property::<SpringConstant>(ch_bond).unwrap().0, 340.0);
    assert_eq!(ff.get_property::<EquilibriumLength>(ch_bond).unwrap().0, 1.09);
}

#[test]
fn query_bonds_by_style() {
    let mut ff = ForceField::new();
    
    let atom_style = ff.create_style("full");
    let c = ff.create_type("C", atom_style);
    let h = ff.create_type("H", atom_style);
    
    let harmonic = ff.create_bond_style("harmonic");
    let morse = ff.create_bond_style("morse");
    
    ff.create_bond_type("C-H_harmonic_1", harmonic, c, h);
    ff.create_bond_type("C-H_harmonic_2", harmonic, c, h);
    ff.create_bond_type("C-H_morse", morse, c, h);
    
    let harmonic_bonds = ff.query_bond_types_of_style(harmonic);
    let morse_bonds = ff.query_bond_types_of_style(morse);
    
    assert_eq!(harmonic_bonds.len(), 2);
    assert_eq!(morse_bonds.len(), 1);
}

#[test]
fn atoms_must_exist_before_bonds() {
    let mut ff = ForceField::new();
    
    let atom_style = ff.create_style("full");
    let bond_style = ff.create_bond_style("harmonic");
    
    // Create atoms FIRST
    let c = ff.create_type("C", atom_style);
    let h = ff.create_type("H", atom_style);
    
    // THEN create bond
    let _ch_bond = ff.create_bond_type("C-H", bond_style, c, h);
    
    // This demonstrates the correct order
    assert_eq!(ff.get_types().len(), 2);
    assert_eq!(ff.get_bond_types().len(), 1);
}

