use wasm_bindgen::prelude::*;
use molcore::core::universe::Universe as CoreUniverse;
use crate::box::JsBox;
use crate::topology::JsTopology;
use crate::entity::JsEntity;

#[wasm_bindgen]
pub struct JsUniverse { pub(crate) inner: CoreUniverse }

#[wasm_bindgen]
impl JsUniverse {
    #[wasm_bindgen(constructor)]
    pub fn new(simbox: &JsBox, topology: &JsTopology) -> JsUniverse {
        JsUniverse { inner: CoreUniverse::new(simbox.inner.clone(), topology.inner.clone()) }
    }
    pub fn empty(simbox: &JsBox) -> JsUniverse { JsUniverse { inner: CoreUniverse::empty(simbox.inner.clone()) } }
    pub fn spawn(&mut self) -> JsEntity { JsEntity::from(self.inner.ecs.spawn()) }
    pub fn despawn(&mut self, e: JsEntity) -> bool { self.inner.ecs.despawn(e.into()) }
    pub fn is_alive(&self, e: JsEntity) -> bool { self.inner.ecs.is_alive(e.into()) }
    pub fn add_atom(&mut self, e: JsEntity) -> bool { self.inner.topology.add_atom(e.into()) }
    pub fn spawn_atom(&mut self) -> JsEntity { let e = self.inner.ecs.spawn(); let _ = self.inner.topology.add_atom(e); e.into() }
}
