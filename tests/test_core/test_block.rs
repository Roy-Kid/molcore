use molomni::core::array::{NdArray, Array, DType};
use molomni::core::Block;

#[test]
fn test_block_insert_and_len0() {
    let mut b = Block::new();
    assert!(b.is_empty());
    assert_eq!(b.len0(), None);

    let a = NdArray::new(vec![3, 2], vec![1i32, 2, 3, 4, 5, 6]);
    b.insert("ints", a).unwrap();
    assert_eq!(b.len0(), Some(3));
    assert_eq!(b.ncols(), 1);

    let f = NdArray::new(vec![3], vec![1.0f64, 2.0, 3.0]);
    b.insert("floats", f).unwrap();
    assert_eq!(b.ncols(), 2);

    let pos = NdArray::new(vec![3, 3], vec![0.0,0.0,0.0, 1.0,1.0,1.0, 2.0,2.0,2.0]);
    b.insert("pos", pos).unwrap();
    assert_eq!(b.ncols(), 3);

    assert_eq!(b.get("floats").unwrap().dtype(), DType::Float64);
    assert_eq!(b.get("ints").unwrap().shape(), &[3, 2]);
}

#[test]
fn test_block_ragged_axis0_rejected() {
    let mut b = Block::new();
    b.insert("a", NdArray::new(vec![4], vec![0i32, 1, 2, 3])).unwrap();
    let bad = NdArray::new(vec![3, 2], vec![0.0f64; 6]);
    let err = b.insert("b", bad).unwrap_err();
    match err {
        molomni::core::block::BlockError::RaggedAxis0 { expected, got, .. } => {
            assert_eq!(expected, 4);
            assert_eq!(got, 3);
        }
        _ => panic!("unexpected error kind"),
    }
}

#[test]
fn test_block_rank_zero_rejected() {
    #[derive(Debug, Clone)]
    struct ScalarArray;
    impl Array for ScalarArray {
        fn dtype(&self) -> DType { DType::Int32 }
        fn shape(&self) -> &[usize] { &[] }
    }

    let mut b = Block::new();
    let err = b.insert("scalar", ScalarArray).unwrap_err();
    match err {
        molomni::core::block::BlockError::RankZero { .. } => {}
        _ => panic!("unexpected error kind"),
    }
}

#[test]
fn test_block_remove_and_reset_len0() {
    let mut b = Block::new();
    b.insert("x", NdArray::new(vec![2], vec![1i32, 2])).unwrap();
    b.insert("y", NdArray::new(vec![2, 3], vec![0.0f64; 6])).unwrap();
    assert_eq!(b.len0(), Some(2));
    b.remove("x");
    assert_eq!(b.len0(), Some(2));
    b.remove("y");
    assert_eq!(b.len0(), None);
    assert!(b.is_empty());
}
