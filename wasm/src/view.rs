use wasm_bindgen::prelude::*;
use ndarray::Array2;

#[wasm_bindgen]
pub struct F32View {
    pub(crate) data: Vec<f32>,
    pub(crate) shape: Vec<usize>,
}

#[wasm_bindgen]
impl F32View {
    #[wasm_bindgen(constructor)]
    pub fn new(shape: Box<[usize]>) -> F32View {
        let shape_vec = shape.to_vec();
        let n: usize = shape_vec.iter().copied().product();
        F32View { data: vec![0.0; n], shape: shape_vec }
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn shape(&self) -> Box<[usize]> { self.shape.clone().into_boxed_slice() }
    pub fn ptr(&mut self) -> *mut f32 { self.data.as_mut_ptr() }
    pub fn to_js_array(&self) -> js_sys::Float32Array { js_sys::Float32Array::from(self.data.as_slice()) }

    /// Fill from a JS Float32Array (copy). Length must match.
    pub fn write_from(&mut self, src: &js_sys::Float32Array) -> Result<(), JsValue> {
        if src.length() as usize != self.data.len() { return Err(JsValue::from_str("source length mismatch")); }
        src.copy_to(&mut self.data[..]);
        Ok(())
    }
}

impl F32View {
    pub(crate) fn as_array2(&self) -> ndarray::Array2<f32> {
        let nrows = self.shape[0];
        ndarray::Array2::from_shape_vec((nrows, 3), self.data.clone()).expect("shape")
    }

    pub(crate) fn from_array2_owned(arr: Array2<f32>) -> F32View {
        let arr_std = arr.as_standard_layout().to_owned();
        let shape = arr_std.shape().to_vec();
        let nrows = shape[0];
        F32View { data: arr_std.into_raw_vec(), shape: vec![nrows, 3] }
    }
}
