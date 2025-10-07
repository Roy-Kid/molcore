use crate::utils::assert_close;
use molcore::core::Element;

#[test]
fn by_symbol_is_case_insensitive() {
    let h1 = Element::by_symbol("H");
    let h2 = Element::by_symbol("h");
    assert_eq!(h1.symbol, "H");
    assert_eq!(h1 as *const _, h2 as *const _);
}

#[test]
fn by_number_works() {
    let h = Element::by_number(1);
    assert_eq!(h.symbol, "H");
}

#[test]
fn element_properties() {
    let h = Element::by_symbol("H");
    assert_eq!(h.z, 1);
    assert_eq!(h.symbol, "H");
    assert_eq!(h.name, "Hydrogen");
    assert_close(h.atomic_mass, 1.008, 1e-3);
}

#[test]
#[should_panic(expected = "invalid symbol")]
fn by_symbol_panics_on_unknown() {
    let _ = Element::by_symbol("Xx");
}

#[test]
#[should_panic(expected = "invalid atomic number")]
fn by_number_panics_on_high_number() {
    let _ = Element::by_number(200);
}

#[test]
fn element_is_copy() {
    let h1 = Element::by_number(1);
    let h2 = *h1;
    assert_eq!(h1.z, h2.z);
}

#[test]
fn element_is_debug() {
    let h = Element::by_number(1);
    let debug_str = format!("{:?}", h);
    assert!(debug_str.contains("Element"));
}
