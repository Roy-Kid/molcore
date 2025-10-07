// Common test utilities

pub fn assert_close(a: f32, b: f32, eps: f32) {
    assert!((a - b).abs() <= eps, "expected {a} ≈ {b} (within {eps})");
}
