use molcore::core::array::DType;

#[test]
fn test_dtype_properties() {
    assert_eq!(DType::Int8.size(), 1);
    assert_eq!(DType::Int16.size(), 2);
    assert_eq!(DType::Int32.size(), 4);
    assert_eq!(DType::Int64.size(), 8);
    assert_eq!(DType::Float32.size(), 4);
    assert_eq!(DType::Float64.size(), 8);
    
    assert_eq!(DType::Int32.name(), "int32");
    assert_eq!(DType::Float64.name(), "float64");
    assert_eq!(DType::Bool.name(), "bool");
}

#[test]
fn test_dtype_equality() {
    let dt1 = DType::Float64;
    let dt2 = DType::Float64;
    let dt3 = DType::Float32;
    
    assert_eq!(dt1, dt2);
    assert_ne!(dt1, dt3);
}

#[test]
fn test_has_dtype_trait() {
    use molcore::core::array::HasDType;
    
    assert_eq!(i32::dtype(), DType::Int32);
    assert_eq!(f64::dtype(), DType::Float64);
    assert_eq!(u8::dtype(), DType::UInt8);
    assert_eq!(bool::dtype(), DType::Bool);
}

#[test]
fn test_dtype_clone_copy() {
    let dt1 = DType::Float64;
    let dt2 = dt1;  // Copy
    let dt3 = dt1.clone();  // Clone
    
    assert_eq!(dt1, dt2);
    assert_eq!(dt1, dt3);
}

#[test]
fn test_all_dtypes() {
    let dtypes = vec![
        DType::Int8,
        DType::Int16,
        DType::Int32,
        DType::Int64,
        DType::UInt8,
        DType::UInt16,
        DType::UInt32,
        DType::UInt64,
        DType::Float32,
        DType::Float64,
        DType::Bool,
    ];
    
    for dtype in dtypes {
        // Ensure all dtypes have valid size and name
        assert!(dtype.size() > 0);
        assert!(!dtype.name().is_empty());
    }
}
