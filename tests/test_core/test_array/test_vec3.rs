use molomni::core::array::{Array, DType, Vec3};

#[test]
fn test_vec3_new() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
    assert_eq!(v.z, 3.0);
}

#[test]
fn test_vec3_clone() {
    let v1 = Vec3::new(1, 2, 3);
    let v2 = v1.clone();
    assert_eq!(v1, v2);
}

#[test]
fn test_vec3_equality() {
    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(1.0, 2.0, 3.0);
    let v3 = Vec3::new(1.0, 2.0, 4.0);
    assert_eq!(v1, v2);
    assert_ne!(v1, v3);
}

#[test]
fn test_vec3_array_trait_shape() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(v.shape(), &[3]);
}

#[test]
fn test_vec3_array_trait_dtype() {
    let v_i32 = Vec3::new(10i32, 20i32, 30i32);
    assert_eq!(v_i32.dtype(), DType::Int32);
    assert_eq!(v_i32.dtype().name(), "int32");
    assert_eq!(v_i32.dtype().size(), 4);
    
    let v_f64 = Vec3::new(1.0f64, 2.0f64, 3.0f64);
    assert_eq!(v_f64.dtype(), DType::Float64);
    assert_eq!(v_f64.dtype().name(), "float64");
    assert_eq!(v_f64.dtype().size(), 8);

    let v_i8 = Vec3::new(1i8, 2i8, 3i8);
    assert_eq!(v_i8.dtype(), DType::Int8);
    assert_eq!(v_i8.dtype().size(), 1);
}

#[test]
fn test_vec3_with_different_types() {
    let v_i32 = Vec3::new(1i32, 2i32, 3i32);
    assert_eq!(v_i32.x, 1i32);
    
    let v_f64 = Vec3::new(1.0f64, 2.0f64, 3.0f64);
    assert_eq!(v_f64.x, 1.0f64);
    
    let v_string = Vec3::new("x".to_string(), "y".to_string(), "z".to_string());
    assert_eq!(v_string.x, "x");
}

#[test]
fn test_vec3_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    
    assert_send::<Vec3<i32>>();
    assert_sync::<Vec3<i32>>();
    assert_send::<Vec3<f64>>();
    assert_sync::<Vec3<f64>>();
}

#[test]
fn test_vec3_debug() {
    let v = Vec3::new(1, 2, 3);
    let debug_str = format!("{:?}", v);
    assert!(debug_str.contains("Vec3"));
    assert!(debug_str.contains("1"));
    assert!(debug_str.contains("2"));
    assert!(debug_str.contains("3"));
}

#[test]
fn test_vec3_different_numeric_types() {
    let v_u8 = Vec3::new(1u8, 2u8, 3u8);
    assert_eq!(v_u8.dtype(), DType::UInt8);
    
    let v_u16 = Vec3::new(1u16, 2u16, 3u16);
    assert_eq!(v_u16.dtype(), DType::UInt16);
    
    let v_u32 = Vec3::new(1u32, 2u32, 3u32);
    assert_eq!(v_u32.dtype(), DType::UInt32);
    
    let v_u64 = Vec3::new(1u64, 2u64, 3u64);
    assert_eq!(v_u64.dtype(), DType::UInt64);
    
    let v_i16 = Vec3::new(1i16, 2i16, 3i16);
    assert_eq!(v_i16.dtype(), DType::Int16);
    
    let v_i64 = Vec3::new(1i64, 2i64, 3i64);
    assert_eq!(v_i64.dtype(), DType::Int64);
    
    let v_f32 = Vec3::new(1.0f32, 2.0f32, 3.0f32);
    assert_eq!(v_f32.dtype(), DType::Float32);
}
