use wasm_bindgen::prelude::*;
use molcore::core::topology::{Topology as CoreTopology, TopoError as CoreTopoError};
use crate::entity::JsEntity;

#[wasm_bindgen]
pub struct JsTopology { pub(crate) inner: CoreTopology }

#[wasm_bindgen]
impl JsTopology {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsTopology { JsTopology { inner: CoreTopology::new() } }

    pub fn add_atom(&mut self, atom: JsEntity) -> bool { self.inner.add_atom(atom.into()) }
    pub fn has_atom(&self, atom: JsEntity) -> bool { self.inner.has_atom(atom.into()) }
    pub fn add_bond(&mut self, a: JsEntity, b: JsEntity) -> Result<bool, JsValue> { self.inner.add_bond(a.into(), b.into()).map_err(to_js_err) }
    pub fn has_bond(&self, a: JsEntity, b: JsEntity) -> bool { self.inner.has_bond(a.into(), b.into()) }
    pub fn add_angle(&mut self, i: JsEntity, j: JsEntity, k: JsEntity) -> Result<bool, JsValue> { self.inner.add_angle(i.into(), j.into(), k.into()).map_err(to_js_err) }
    pub fn neighbors(&self, a: JsEntity) -> Box<[JsEntity]> {
        self.inner.neighbors(a.into()).map(|set| set.iter().copied().map(Into::into).collect::<Vec<_>>().into_boxed_slice()).unwrap_or_else(|| Box::from([]))
    }
    pub fn atom_count(&self) -> usize { self.inner.atom_count() }
    pub fn bond_count(&self) -> usize { self.inner.bond_count() }
    pub fn angle_count(&self) -> usize { self.inner.angle_count() }
}

fn to_js_err(e: CoreTopoError) -> JsValue { JsValue::from_str(&e.to_string()) }
