use molomni::core::{Frame, Block};
use molomni::core::array::{NdArray, Array, DType};

#[test]
fn test_frame_basic_ops() {
    let mut f = Frame::new();
    assert!(f.is_empty());
    assert_eq!(f.len(), 0);

    // Block A with 2 rows
    let mut a = Block::new();
    a.insert("x", NdArray::new(vec![2], vec![1i32, 2])).unwrap();
    a.insert("y", NdArray::new(vec![2, 3], vec![0.0f64; 6])).unwrap();
    assert_eq!(a.nrows(), Some(2));
    assert_eq!(a.len(), 2);

    // Block B with 3 rows
    let mut b = Block::new();
    b.insert("pos", NdArray::new(vec![3, 3], vec![
        0.0,0.0,0.0,
        1.0,1.0,1.0,
        2.0,2.0,2.0,
    ])).unwrap();
    assert_eq!(b.nrows(), Some(3));
    assert_eq!(b.len(), 1);

    // Insert both into frame
    assert!(f.insert("A", a).is_none());
    assert!(f.insert("B", b).is_none());
    assert_eq!(f.len(), 2);
    assert!(f.contains_key("A") && f.contains_key("B"));

    // Access and inspect
    let ba = f.get("A").unwrap();
    assert_eq!(ba.nrows(), Some(2));
    assert_eq!(ba.len(), 2);

    // Replace and remove
    let mut c = Block::new();
    c.insert("z", NdArray::new(vec![1], vec![42i32])).unwrap();
    let prev = f.insert("A", c).unwrap();
    assert_eq!(prev.nrows(), Some(2));
    assert_eq!(f.len(), 2);

    let removed = f.remove("B").unwrap();
    assert_eq!(removed.nrows(), Some(3));
    assert_eq!(f.len(), 1);

    f.clear();
    assert!(f.is_empty());
}
