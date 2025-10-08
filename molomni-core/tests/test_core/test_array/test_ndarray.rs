use molomni::core::array::{Array, DType, NdArray};

#[test]
fn test_ndarray_new_and_shape_dtype() {
    let a = NdArray::new(vec![2, 3], vec![1i32, 2, 3, 4, 5, 6]);
    assert_eq!(a.shape(), &[2, 3]);
    assert_eq!(a.dtype(), DType::Int32);
}

#[test]
fn test_ndarray_get_and_indexing_2d() {
    // 2x3: rows [[1,2,3],[4,5,6]]
    let a = NdArray::new(vec![2, 3], vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(a.get(&[0, 0]), Some(&1));
    assert_eq!(a.get(&[0, 2]), Some(&3));
    assert_eq!(a.get(&[1, 0]), Some(&4));
    assert_eq!(a.get(&[1, 2]), Some(&6));
    assert!(a.get(&[2, 0]).is_none());

    // Index operator (panics on OOB)
    assert_eq!(a[[1, 2]], 6);
}

#[test]
fn test_ndarray_mut_indexing() {
    let mut a = NdArray::new(vec![2, 2, 2], vec![0i32; 8]);
    a[[1, 0, 1]] = 42;
    assert_eq!(a.get(&[1, 0, 1]), Some(&42));
}

#[test]
fn test_ndarray_generic_array_trait() {
    fn check<T: Array>(a: &T) -> (DType, Vec<usize>) { (a.dtype(), a.shape().to_vec()) }
    let a = NdArray::new(vec![3], vec![1.0f64, 2.0, 3.0]);
    let (dt, shp) = check(&a);
    assert_eq!(dt, DType::Float64);
    assert_eq!(shp, vec![3]);
}

#[test]
fn test_array_macro_1d_and_2d() {
    let a1 = molomni::array![1, 2, 3, 4];
    assert_eq!(a1.shape(), &[4]);
    assert_eq!(a1.dtype(), DType::Int32);
    assert_eq!(a1[[2]], 3);

    let a2 = molomni::array![[10., 20., 30., 40.], [50., 60., 70., 80.]];
    assert_eq!(a2.shape(), &[2, 4]);
    assert_eq!(a2.dtype(), DType::Float64);
    assert_eq!(a2[[0, 3]], 40.0);
    assert_eq!(a2[[1, 0]], 50.0);
}

#[test]
fn test_range_i32_and_f64() {
    let r_i32 = NdArray::<i32>::range(0, 6, 2);
    assert_eq!(r_i32.shape(), &[3]);
    assert_eq!(r_i32.dtype(), DType::Int32);
    assert_eq!(r_i32.data(), &vec![0, 2, 4]);

    // numpy-like: default integer, then astype to float
    let r = NdArray::<i32>::arange(0, 4, 1);
    assert_eq!(r.data(), &vec![0, 1, 2, 3]);
    let r_f64 = r.astype::<f64>();
    assert_eq!(r_f64.dtype(), DType::Float64);
    let expected = vec![0.0, 1.0, 2.0, 3.0];
    for i in 0..4 { assert!((r_f64[[i]] - expected[i]).abs() < 1e-12); }
}
