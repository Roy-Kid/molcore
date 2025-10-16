use wasm_bindgen::prelude::*;
use molcore::core::region::r#box::Box as CoreBox;
use ndarray::Array2;

use crate::view::F32View;

#[wasm_bindgen]
pub struct JsBox { pub(crate) inner: CoreBox }

#[wasm_bindgen]
impl JsBox {
    #[wasm_bindgen(constructor)]
    pub fn new(h: Box<[f32]>, origin: Box<[f32]>, pbc_x: bool, pbc_y: bool, pbc_z: bool) -> Result<JsBox, JsValue> {
        if h.len() != 9 || origin.len() != 3 { return Err(JsValue::from_str("invalid shapes")); }
        let h_mat = ndarray::arr2(&[
            [h[0], h[1], h[2]],
            [h[3], h[4], h[5]],
            [h[6], h[7], h[8]],
        ]);
        let origin = ndarray::arr1(&[origin[0], origin[1], origin[2]]);
        let inner = CoreBox::new(h_mat, origin, [pbc_x, pbc_y, pbc_z]);
        Ok(JsBox { inner })
    }

    pub fn cube(a: f32, origin: Box<[f32]>, pbc_x: bool, pbc_y: bool, pbc_z: bool) -> Result<JsBox, JsValue> {
        if origin.len() != 3 { return Err(JsValue::from_str("invalid origin")); }
        let inner = CoreBox::cube(a, ndarray::arr1(&[origin[0], origin[1], origin[2]]), [pbc_x, pbc_y, pbc_z]);
        Ok(JsBox { inner })
    }

    pub fn ortho(lengths: Box<[f32]>, origin: Box<[f32]>, pbc_x: bool, pbc_y: bool, pbc_z: bool) -> Result<JsBox, JsValue> {
        if lengths.len() != 3 || origin.len() != 3 { return Err(JsValue::from_str("invalid args")); }
        let inner = CoreBox::ortho(ndarray::arr1(&[lengths[0], lengths[1], lengths[2]]), ndarray::arr1(&[origin[0], origin[1], origin[2]]), [pbc_x, pbc_y, pbc_z]);
        Ok(JsBox { inner })
    }

    pub fn volume(&self) -> f32 { self.inner.volume() }
    pub fn lengths(&self) -> Box<[f32]> { self.inner.lengths().to_vec().into_boxed_slice() }
    pub fn tilts(&self) -> Box<[f32]> { self.inner.tilts().to_vec().into_boxed_slice() }

    pub fn to_frac(&self, view: &F32View) -> crate::view::F32View {
        let out = self.inner.to_frac(view.as_array2().view());
        crate::view::F32View::from_array2_owned(out)
    }
    pub fn to_cart(&self, view: &F32View) -> crate::view::F32View {
        let out = self.inner.to_cart(view.as_array2().view());
        crate::view::F32View::from_array2_owned(out)
    }
    pub fn wrap(&self, view: &F32View) -> crate::view::F32View {
        let out = self.inner.wrap(view.as_array2().view());
        crate::view::F32View::from_array2_owned(out)
    }
    pub fn delta(&self, a: &F32View, b: &F32View, minimum_image: bool) -> crate::view::F32View {
        let out = self.inner.delta(a.as_array2().view(), b.as_array2().view(), minimum_image);
        crate::view::F32View::from_array2_owned(out)
    }
}
