/// Construct an NdArray with NumPy-like syntax.
///
/// Supports 1D and 2D literals with trailing commas:
/// - 1D: `array![1, 2, 3]`
/// - 2D: `array![[1, 2], [3, 4]]`
#[macro_export]
macro_rules! array {
    // 顶层：若干子数组（无需再包一层[]）-> 递归入口
    ( $( [ $($inner:tt)* ] ),+ $(,)? ) => {{
        $crate::array!(@nd $( [ $($inner)* ] ),+ )
    }};

    // 1D（带一层[]）
    ( [ $( $x:expr ),+ $(,)? ] ) => {{
        let v = vec![ $( $x ),* ];
        $crate::core::array::NdArray::new(vec![v.len()], v)
    }};

    // 1D（无括号，兼容写法）：转发到带括号分支
    ( $( $x:expr ),+ $(,)? ) => {{
        let v = vec![ $( $x ),* ];
        $crate::core::array::NdArray::new(vec![v.len()], v)
    }};

    // 空数组
    ( [ ] ) => {{
        $crate::core::array::NdArray::<i32>::new(vec![0], Vec::new())
    }};

    // 内部：递归构造高维
    ( @nd $( [ $($inner:tt)* ] ),+ $(,)? ) => {{
        let children = vec![ $( $crate::array!(@dispatch [ $($inner)* ]) ),+ ];
        $crate::core::array::NdArray::stack(children).expect("ragged shape in array! macro")
    }};

    // 内部分发：优先识别更高维，否则当作 1D 叶子
    ( @dispatch [ $( [ $($inner:tt)* ] ),+ $(,)? ] ) => {{
        $crate::array!(@nd $( [ $($inner)* ] ),+ )
    }};
    ( @dispatch [ $( $x:expr ),+ $(,)? ] ) => {{
        let v = vec![ $( $x ),* ];
        $crate::core::array::NdArray::new(vec![v.len()], v)
    }};
}