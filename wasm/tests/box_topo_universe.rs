use wasm_bindgen_test::*;
use molwasm::{F32View, JsBox, JsTopology, JsUniverse, JsEntity, parse_xyz_frame};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn box_to_frac_to_cart_zero_copy() {
    // Prepare a 2x3 positions view
    let mut view = F32View::new(Box::new([2usize, 3usize]));
    // write via JS array shim
    let js = js_sys::Float32Array::from(vec![0.0, 0.0, 0.0, 2.0, 3.0, 4.0].as_slice());
    view.write_from(&js).unwrap();

    // Make a simple ortho box
    let bx = JsBox::ortho(Box::new([2.0, 3.0, 4.0]), Box::new([0.0, 0.0, 0.0]), true, true, true).unwrap();

    // frac -> cart -> frac roundtrip via zero-copy interface
    let frac = bx.to_frac(&view);
    // first row should be 0,0,0 ; second (1,1,1)
    let f = frac.to_js_array();
    assert_eq!(f.get_index(0), 0.0);
    assert_eq!(f.get_index(4), 1.0);

    let cart = bx.to_cart(&frac);
    let c = cart.to_js_array();
    assert_eq!(c.get_index(0), 0.0);
    assert_eq!(c.get_index(1), 0.0);
    assert_eq!(c.get_index(2), 0.0);
    assert_eq!(c.get_index(3), 2.0);
    assert_eq!(c.get_index(4), 3.0);
    assert_eq!(c.get_index(5), 4.0);
}

#[wasm_bindgen_test]
fn topology_and_universe_basic() {
    let topo = JsTopology::new();
    let bx = JsBox::cube(10.0, Box::new([0.0, 0.0, 0.0]), true, true, true).unwrap();
    let mut uni = JsUniverse::empty(&bx);

    // spawn and add atoms
    let a = uni.spawn();
    let b = uni.spawn();
    assert!(uni.is_alive(a));
    assert!(uni.is_alive(b));
    assert!(uni.add_atom(a));
    assert!(uni.add_atom(b));

    // bond
    assert!(uni.inner.topology.add_bond(a.into(), b.into()).unwrap());
    assert!(uni.inner.topology.has_bond(a.into(), b.into()));

    // despawn
    assert!(uni.despawn(a));
    assert!(!uni.is_alive(a));
}
