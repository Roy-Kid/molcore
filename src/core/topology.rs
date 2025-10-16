use std::collections::{HashMap, HashSet};
use super::ecs::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bond(pub Entity, pub Entity);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Angle(pub Entity, pub Entity, pub Entity); // (i, j, k) where j is the center

#[derive(Debug)]
pub enum TopoError {
    AtomNotFound(Entity),
    BondSelfLoop,
    AngleMissingBond,
    AngleNotDistinct,
    DihMissingBond,
    DihNotDistinct,
}

impl std::fmt::Display for TopoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopoError::AtomNotFound(e) => write!(f, "atom {:?} not found", e),
            TopoError::BondSelfLoop => write!(f, "bond requires distinct atoms"),
            TopoError::AngleMissingBond => write!(f, "angle requires existing bonds (i-j) & (j-k)"),
            TopoError::AngleNotDistinct => write!(f, "angle nodes must be distinct"),
            TopoError::DihMissingBond => write!(f, "dihedral requires bonds (i-j),(j-k),(k-l)"),
            TopoError::DihNotDistinct => write!(f, "dihedral nodes must be distinct"),
        }
    }
}

impl std::error::Error for TopoError {}

fn ordered_pair(a: Entity, b: Entity) -> (Entity, Entity) {
    if a.0 <= b.0 { (a, b) } else { (b, a) }
}

fn canonical_angle(i: Entity, j: Entity, k: Entity) -> (Entity, Entity, Entity) {
    // Canonicalize by ordering the ends (i,k), keeping j as center
    if i.0 <= k.0 { (i, j, k) } else { (k, j, i) }
}

#[derive(Debug, Clone)]
pub struct Topology {
    atoms: HashSet<Entity>,
    bonds: HashSet<(Entity, Entity)>, // stored as ordered pair (min,max)
    angles: HashSet<(Entity, Entity, Entity)>, // stored as canonical (min(i,k), j, max(i,k))
    adj: HashMap<Entity, HashSet<Entity>>, // adjacency list for quick neighbor lookups
}

impl Topology {
    pub fn new() -> Self {
        Self {
            atoms: HashSet::new(),
            bonds: HashSet::new(),
            angles: HashSet::new(),
            adj: HashMap::new(),
        }
    }

    pub fn add_atom(&mut self, atom: Entity) -> bool {
        let inserted = self.atoms.insert(atom);
        self.adj.entry(atom).or_insert_with(HashSet::new);
        inserted
    }

    pub fn has_atom(&self, atom: Entity) -> bool { self.atoms.contains(&atom) }

    pub fn add_bond(&mut self, a: Entity, b: Entity) -> Result<bool, TopoError> {
        if a == b { return Err(TopoError::BondSelfLoop); }
        if !(self.has_atom(a) && self.has_atom(b)) { return Err(TopoError::AtomNotFound(if !self.has_atom(a) { a } else { b })); }
        let key = ordered_pair(a, b);
        let inserted = self.bonds.insert(key);
        if inserted {
            self.adj.entry(a).or_default().insert(b);
            self.adj.entry(b).or_default().insert(a);
        }
        Ok(inserted)
    }

    pub fn has_bond(&self, a: Entity, b: Entity) -> bool { self.bonds.contains(&ordered_pair(a, b)) }

    pub fn add_angle(&mut self, i: Entity, j: Entity, k: Entity) -> Result<bool, TopoError> {
        if i == j || j == k || i == k { return Err(TopoError::AngleNotDistinct); }
        if !(self.has_atom(i) && self.has_atom(j) && self.has_atom(k)) {
            // return first missing atom for clarity
            let missing = if !self.has_atom(i) { i } else if !self.has_atom(j) { j } else { k };
            return Err(TopoError::AtomNotFound(missing));
        }
        if !self.has_bond(i, j) || !self.has_bond(j, k) { return Err(TopoError::AngleMissingBond); }
        let key = canonical_angle(i, j, k);
        Ok(self.angles.insert(key))
    }

    pub fn neighbors(&self, a: Entity) -> Option<&HashSet<Entity>> { self.adj.get(&a) }

    pub fn atom_count(&self) -> usize { self.atoms.len() }
    pub fn bond_count(&self) -> usize { self.bonds.len() }
    pub fn angle_count(&self) -> usize { self.angles.len() }
}