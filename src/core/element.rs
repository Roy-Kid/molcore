//! Element data and basic lookup utilities.
use core::str::FromStr;

/// Chemical element (partial table)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Element {
    /// Hydrogen
    H,
    // He,
    // Li,
    // ...
}

impl Element {
    /// Atomic number (Z)
    pub const fn z(self) -> u8 {
        match self {
            Element::H => 1,
        }
    }

    /// Chemical symbol (e.g., "H")
    pub const fn symbol(self) -> &'static str {
        match self {
            Element::H => "H",
        }
    }

    /// English element name
    pub const fn name(self) -> &'static str {
        match self {
            Element::H => "Hydrogen",
        }
    }

    /// Standard atomic mass (approx.)
    pub const fn atomic_mass(self) -> f32 {
        match self {
            Element::H => 1.008,
        }
    }

    /// All supported elements
    pub const ALL: &'static [Element] = &[
        Element::H,
    ];

    /// Lookup by atomic number
    pub fn by_number(z: u8) -> Option<Element> {
        Self::ALL.iter().copied().find(|e| e.z() == z)
    }

    /// Lookup by symbol (case-insensitive)
    pub fn by_symbol(sym: &str) -> Option<Element> {
        Self::ALL
            .iter()
            .copied()
            .find(|e| e.symbol().eq_ignore_ascii_case(sym))
    }
}

impl FromStr for Element {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Element::by_symbol(s).ok_or(())
    }
}

impl core::fmt::Display for Element {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.symbol())
    }
}

#[cfg(test)]
mod test_element {

    use super::*;

    #[test]
    fn test_accessors() {
        assert_eq!(Element::by_number(1), Some(Element::H));
        assert_eq!(Element::by_symbol("H"), Some(Element::H));
        assert_eq!(Element::by_symbol("h"), Some(Element::H));
        assert_eq!(Element::by_symbol("X"), None);

    }

    #[test]
    fn test_props() {
        let e = Element::H;
        assert_eq!(e.z(), 1);
        assert_eq!(e.symbol(), "H");
        assert_eq!(e.name(), "Hydrogen");
        assert_eq!(e.atomic_mass(), 1.008);
        assert_eq!(format!("{}", e), "H");
    }

}