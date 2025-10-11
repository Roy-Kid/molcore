use wasm_bindgen_test::*;
use molwasm::{parse_xyz_frame, JsFrame};
use molwasm::F32View;
use serde_wasm_bindgen as swb;

#[wasm_bindgen_test]
fn parse_plain_xyz() {
    let s = "2\nWater molecule\nO 0.0 0.0 0.0\nH 1.0 0.0 0.0\n";
    let v = parse_xyz_frame(s).expect("parse ok");
    let frame: JsFrame = swb::from_value(v).expect("serde from_value");
    assert_eq!(frame.atoms.nrows, 2);
    // columns should include x,y,z (plain xyz)
    assert!(frame.atoms.columns.contains(&"x".to_string()));
    assert!(frame.atoms.columns.contains(&"y".to_string()));
    assert!(frame.atoms.columns.contains(&"z".to_string()));
    assert!(frame.atoms.data.len() >= 3);
}

#[wasm_bindgen_test]
fn parse_extxyz_with_properties() {
    let s = "3\nProperties=species:S:1:pos:R:3\nH 0 0 1\nO 0 1 0\nH 1 0 0\n";
    let v = parse_xyz_frame(s).expect("parse ok");
    let frame: JsFrame = swb::from_value(v).expect("serde from_value");
    assert_eq!(frame.atoms.nrows, 3);
    assert!(frame.atoms.data.len() >= 3);
}

#[wasm_bindgen_test]
fn f32view_roundtrip_and_sum() {
    // 2x3 buffer
    let mut view = F32View::new(Box::new([2usize, 3usize]));
    assert_eq!(view.len(), 6);
    let ptr = view.ptr() as u32; // ptr is tested for non-zero
    assert!(ptr != 0);

    // Fill via write_from copy path
    let js_arr = js_sys::Float32Array::from(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0].as_slice());
    view.write_from(&js_arr).unwrap();
    assert_eq!(view.sum(), 21.0);

    // Check to_js_array roundtrip
    let back = view.to_js_array();
    assert_eq!(back.length(), 6);
    assert_eq!(back.get_index(0), 1.0);
    assert_eq!(back.get_index(5), 6.0);
}
