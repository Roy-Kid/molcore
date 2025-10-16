use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use ndarray::{s, Array1};
use molcore::io::xyz::parse_xyz_frame_str;

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
    use molcore::core::frame::Frame;
    use molcore::core::block::Block;
    let frame: Frame = parse_xyz_frame_str(s).map_err(|e| JsValue::from_str(&e))?;
    let atoms: &Block = frame.get("atoms").ok_or_else(|| JsValue::from_str("no atoms block"))?;
    let nrows = atoms.nrows().unwrap_or(0);

    let mut columns: Vec<String> = Vec::new();
    let mut data: Vec<Vec<f32>> = Vec::new();
    for (name, arr) in atoms.iter() {
        let shape = arr.shape();
        if shape.len() == 2 && shape[0] == nrows && shape[1] == 1 {
            let a2 = arr.view().into_dimensionality::<ndarray::Ix2>()
                .map_err(|_| JsValue::from_str("expected 2D array"))?;
            let col: Array1<f32> = a2.slice(s![.., 0]).to_owned().into_dimensionality::<ndarray::Ix1>().unwrap();
            columns.push(name.to_string());
            data.push(col.to_vec());
        } else if shape.len() == 2 && shape[0] == nrows && shape[1] > 1 {
            let a2 = arr.view().into_dimensionality::<ndarray::Ix2>()
                .map_err(|_| JsValue::from_str("expected 2D array"))?;
            for j in 0..shape[1] {
                let col: Array1<f32> = a2.slice(s![.., j]).to_owned().into_dimensionality::<ndarray::Ix1>().unwrap();
                columns.push(format!("{}_{j}", name));
                data.push(col.to_vec());
            }
        }
    }

    let js = JsFrame { meta: frame.meta, atoms: JsBlock { nrows, columns, data } };
    serde_wasm_bindgen::to_value(&js).map_err(|e| JsValue::from_str(&e.to_string()))
}
