//! Element data and lookup functionality.

/// Represents a chemical element with its fundamental properties.
///
/// # Examples
///
/// ```
/// use molomni::core::Element;
///
/// let hydrogen = Element::by_number(1);
/// assert_eq!(hydrogen.symbol, "H");
/// assert_eq!(hydrogen.name, "Hydrogen");
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Element {
    /// Atomic number (number of protons)
    pub z: u8,
    /// Chemical symbol (e.g., "H", "He", "Li")
    pub symbol: &'static str,
    /// Full element name (e.g., "Hydrogen")
    pub name: &'static str,
    /// Atomic mass in unified atomic mass units (u)
    pub atomic_mass: f32,
}

impl Element {
    /// Finds an element by its atomic number.
    ///
    /// # Arguments
    ///
    /// * `z` - The atomic number to search for
    ///
    /// # Returns
    ///
    /// A static reference to the `Element` with the given atomic number.
    ///
    /// # Panics
    ///
    /// Panics with "invalid atomic number" if no element with the given atomic number exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use molomni::core::Element;
    ///
    /// let hydrogen = Element::by_number(1);
    /// assert_eq!(hydrogen.symbol, "H");
    /// ```
    pub fn by_number(z: u8) -> &'static Element {
        ELEMENTS
            .iter()
            .find(|e| e.z == z)
            .expect("invalid atomic number")
    }

    /// Finds an element by its chemical symbol (case-insensitive).
    ///
    /// # Arguments
    ///
    /// * `sym` - The chemical symbol to search for (e.g., "H", "h", "He")
    ///
    /// # Returns
    ///
    /// A static reference to the `Element` with the given symbol.
    ///
    /// # Panics
    ///
    /// Panics with "invalid symbol" if no element with the given symbol exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use molomni::core::Element;
    ///
    /// let h1 = Element::by_symbol("H");
    /// let h2 = Element::by_symbol("h");
    /// assert_eq!(h1 as *const _, h2 as *const _); // Same element
    /// ```
    pub fn by_symbol(sym: &str) -> &'static Element {
        ELEMENTS
            .iter()
            .find(|e| e.symbol.eq_ignore_ascii_case(sym))
            .expect("invalid symbol")
    }
}

/// Static array containing all known chemical elements.
///
/// Currently contains only a subset of elements. More will be added in future versions.
pub static ELEMENTS: &[Element] = &[Element {
    z: 1,
    symbol: "H",
    name: "Hydrogen",
    atomic_mass: 1.008,
}];
