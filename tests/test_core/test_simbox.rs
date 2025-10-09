use molomni::core::array::{Mat3, NdArray};
use molomni::core::{SimBox, Vec3};

#[test]
fn test_to_frac_to_cart_roundtrip() {
    let h = Mat3::from_rows([2.0, 0.5, 0.0], [0.0, 2.0, 0.3], [0.0, 0.0, 3.0]);
    let boxx = SimBox::new(h, Vec3::new(0.1, -0.2, 0.3), Vec3::new(true, true, true));
    let cart = NdArray::new(vec![3], vec![1.3, -0.7, 2.2]);
    let frac = boxx.to_frac(&cart);
    let cart2 = boxx.to_cart(&frac);
    for i in 0..3 { assert!((cart[[i]] - cart2[[i]]).abs() < 1e-6); }
}

#[test]
fn test_wrap() {
    let h = Mat3::from_rows([3.0, 0.0, 0.0], [0.2, 2.0, 0.0], [0.0, 0.1, 1.5]);
    let boxx = SimBox::new(h, Vec3::new(0.0, 0.0, 0.0), Vec3::new(true, true, true));
    let pts = NdArray::new(vec![3], vec![3.2, -0.5, 1.8]);
    let w = boxx.wrap(&pts);
    // Result should lie inside primary cell in fractional coords
    let f = boxx.to_frac(&w);
    for i in 0..3 { assert!(f[[i]] >= 0.0 && f[[i]] < 1.0); }
}

#[test]
fn test_delta_minimum_image() {
    let h = Mat3::from_rows([2.0, 0.3, 0.0], [0.0, 2.5, 0.4], [0.0, 0.0, 2.0]);
    let boxx = SimBox::new(h, Vec3::new(0.0, 0.0, 0.0), Vec3::new(true, true, true));
    let a = NdArray::new(vec![3], vec![0.1, 0.1, 0.1]);
    let b = NdArray::new(vec![3], vec![1.9, 0.1, 0.1]);
    let d = boxx.delta(&a, &b, true);
    // along x, should wrap to -0.2 approximately in Cartesian (depends on skew, check magnitude small)
    assert!(d[[0]].abs() < 0.5);
}

#[test]
fn test_volume() {
    let h = Mat3::from_rows([2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]);
    let boxx = SimBox::new(h, Vec3::new(0.0, 0.0, 0.0), Vec3::new(true, true, true));
    assert!((boxx.volume() - 24.0).abs() < 1e-6);
}
