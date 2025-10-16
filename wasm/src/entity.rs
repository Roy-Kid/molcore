use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use molcore::core::ecs::Entity as CoreEntity;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct JsEntity { pub index: u32, pub generation: u32 }

impl From<CoreEntity> for JsEntity { fn from(e: CoreEntity) -> Self { JsEntity { index: e.index() as u32, generation: e.generation() } } }
impl From<JsEntity> for CoreEntity { fn from(j: JsEntity) -> Self { CoreEntity(((j.generation as u64) << 32) | (j.index as u64)) } }
