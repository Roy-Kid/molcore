use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use molcore::core::array::NdArray;
use molcore::io::xyz::parse_xyz_frame_str;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// ===== XYZ parser binding =====

#[derive(Debug, Serialize, Deserialize)]
pub struct JsBlock {
    pub nrows: usize,
    pub columns: Vec<String>,
    pub data: Vec<Vec<f32>>, // per-column data (skip string columns)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsFrame {
    pub meta: std::collections::HashMap<String, String>,
    pub atoms: JsBlock,
}

#[wasm_bindgen]
pub fn parse_xyz_frame(s: &str) -> Result<JsValue, JsValue> {
    let frame = parse_xyz_frame_str(s).map_err(|e| JsValue::from_str(&e))?;
    let atoms = frame.get("atoms").ok_or_else(|| JsValue::from_str("no atoms block"))?;
    let nrows = atoms.nrows().unwrap_or(0);

    // Collect numeric/boolean columns
    let mut columns: Vec<String> = Vec::new();
    let mut data: Vec<Vec<f32>> = Vec::new();
    for (name, arr) in atoms.iter() {
        use molcore::core::array::DType;
        match arr.dtype() {
            DType::Float32 => {
                columns.push(name.to_string());
                let a = arr.as_any().downcast_ref::<NdArray<f32>>().ok_or_else(|| JsValue::from_str("downcast f32 failed"))?;
                data.push(a.data().to_vec());
            }
            DType::Int32 => {
                columns.push(name.to_string());
                let a = arr.as_any().downcast_ref::<NdArray<i32>>().ok_or_else(|| JsValue::from_str("downcast i32 failed"))?;
                data.push(a.data().iter().map(|&x| x as f32).collect());
            }
            DType::Int64 => {
                columns.push(name.to_string());
                let a = arr.as_any().downcast_ref::<NdArray<i64>>().ok_or_else(|| JsValue::from_str("downcast i64 failed"))?;
                data.push(a.data().iter().map(|&x| x as f32).collect());
            }
            DType::Bool => {
                columns.push(name.to_string());
                let a = arr.as_any().downcast_ref::<NdArray<bool>>().ok_or_else(|| JsValue::from_str("downcast bool failed"))?;
                data.push(a.data().iter().map(|&b| if b {1.0} else {0.0}).collect());
            }
            _ => { /* skip unsupported types for now */ }
        }
    }

    let js = JsFrame { meta: frame.meta, atoms: JsBlock { nrows, columns, data } };
    serde_wasm_bindgen::to_value(&js).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ===== Minimal ndarray-like view for f32 backed by wasm memory =====

#[wasm_bindgen]
pub struct F32View {
    data: Vec<f32>,
    shape: Vec<usize>,
}

#[wasm_bindgen]
impl F32View {
    /// Allocate an f32 buffer with the given shape (row-major). Data initialized to 0.
    #[wasm_bindgen(constructor)]
    pub fn new(shape: Box<[usize]>) -> F32View {
        let shape_vec = shape.to_vec();
        let n: usize = shape_vec.iter().copied().product();
        F32View { data: vec![0.0; n], shape: shape_vec }
    }

    /// Number of elements in the buffer
    pub fn len(&self) -> usize { self.data.len() }

    /// Shape vector
    pub fn shape(&self) -> Box<[usize]> { self.shape.clone().into_boxed_slice() }

    /// Raw pointer (byte offset) to the start of the buffer for zero-copy JS writes.
    /// Usage from JS: new Float32Array(wasmMemory.buffer, ptr, len)
    pub fn ptr(&mut self) -> *mut f32 { self.data.as_mut_ptr() }

    /// Fill from a JS Float32Array (copy)
    pub fn write_from(&mut self, src: &js_sys::Float32Array) -> Result<(), JsValue> {
        if src.length() as usize != self.data.len() {
            return Err(JsValue::from_str("source length mismatch"));
        }
        src.copy_to(&mut self.data[..]);
        Ok(())
    }

    /// Read out as a JS Float32Array (copy)
    pub fn to_js_array(&self) -> js_sys::Float32Array {
        js_sys::Float32Array::from(self.data.as_slice())
    }

    /// Sum of all elements (via simple Rust loop)
    pub fn sum(&self) -> f32 { self.data.iter().copied().sum() }

    /// Sum via NdArray wrapper (demonstrates constructing an NdArray view)
    pub fn sum_via_ndarray(&self) -> f32 {
        let arr = NdArray::from_vec(self.shape.clone(), self.data.clone());
        arr.data().iter().copied().sum()
    }
}
