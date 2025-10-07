use molcore::core::array::{Array, DType, Vec3};

#[test]
fn test_array_trait_on_vec3() {
    let v = Vec3::new(1.0f64, 2.0f64, 3.0f64);
    
    // Test that Vec3 implements Array trait
    assert_eq!(v.dtype(), DType::Float64);
    assert_eq!(v.shape(), &[3]);
}

#[test]
fn test_array_trait_with_generic_function() {
    fn check_array<T: Array>(arr: &T) -> (DType, Vec<usize>) {
        (arr.dtype(), arr.shape().to_vec())
    }
    
    let v_i32 = Vec3::new(1i32, 2i32, 3i32);
    let (dtype, shape) = check_array(&v_i32);
    assert_eq!(dtype, DType::Int32);
    assert_eq!(shape, vec![3]);
    
    let v_f32 = Vec3::new(1.0f32, 2.0f32, 3.0f32);
    let (dtype, shape) = check_array(&v_f32);
    assert_eq!(dtype, DType::Float32);
    assert_eq!(shape, vec![3]);
}

#[test]
fn test_array_trait_send_sync() {
    fn requires_array<T: Array>(_: T) {}
    
    let v = Vec3::new(1, 2, 3);
    requires_array(v);
}
