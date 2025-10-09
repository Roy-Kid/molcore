

use std::collections::HashMap;
use polars::prelude::DataFrame;

#[derive(Debug, Clone)]
pub enum MetaType {
    Int64(i64),
    Float64(f64),
    String(String),
    Bool(bool),
}

pub trait MapLike<T> {
    fn get(&self, key: &str) -> Option<T>;
    fn set(&mut self, key: &str, value: T);
}

pub type DefaultMeta = HashMap<String, MetaType>;

#[derive(Clone, Debug, Default)]
pub struct Frame<M = DefaultMeta> {
    components: std::collections::HashMap<String, DataFrame>,
    pub meta: M,
}

impl<M: Default> Frame<M> {
    #[inline]
    pub fn new() -> Self { Self::default() }
}

impl<M> Frame<M> {
    /// 插入或替换一个组件（不校验）
    #[inline]
    pub fn insert(&mut self, key: impl Into<String>, df: DataFrame) {
        self.components.insert(key.into(), df);
    }

    /// 获取不可变引用（不存在则 None）
    #[inline]
    pub fn get(&self, key: &str) -> Option<&DataFrame> {
        self.components.get(key)
    }

    /// 获取可变引用（不存在则 None）
    #[inline]
    pub fn get_mut(&mut self, key: &str) -> Option<&mut DataFrame> {
        self.components.get_mut(key)
    }

    /// 移除并返回组件
    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<DataFrame> {
        self.components.remove(key)
    }

    /// 是否包含某组件
    #[inline]
    pub fn contains(&self, key: &str) -> bool {
        self.components.contains_key(key)
    }

    /// 列出所有键
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.components.keys().map(|s| s.as_str())
    }

    /// 返回组件总数
    #[inline]
    pub fn len(&self) -> usize { self.components.len() }

    /// 是否为空
    #[inline]
    pub fn is_empty(&self) -> bool { self.components.is_empty() }

    /// 只读访问所有组件
    #[inline]
    pub fn components(&self) -> &std::collections::HashMap<String, DataFrame> { &self.components }

    /// 可变访问所有组件（请自担一致性）
    #[inline]
    pub fn components_mut(&mut self) -> &mut std::collections::HashMap<String, DataFrame> { &mut self.components }
}

pub trait HasTimestep {
    fn timestep(&self) -> Option<i64>;
    fn set_timestep(&mut self, v: i64);
}


pub trait HasBoxH {
    fn box_h(&self) -> Option<[[f64; 3]; 3]>;
    fn set_box_h(&mut self, h: [[f64; 3]; 3]);
}

impl<M: MapLike<i64>> HasTimestep for Frame<M> {
    fn timestep(&self) -> Option<i64> { self.meta.get("timestep") }
    fn set_timestep(&mut self, v: i64) { self.meta.set("timestep", v) }
}

impl<M: MapLike<[[f64; 3]; 3]>> HasBoxH for Frame<M> {
    fn box_h(&self) -> Option<[[f64; 3]; 3]> { self.meta.get("box") }
    fn set_box_h(&mut self, h: [[f64; 3]; 3]) { self.meta.set("box", h) }
}

// Implement MapLike for DefaultMeta (HashMap<String, MetaType>)
impl MapLike<i64> for DefaultMeta {
    fn get(&self, key: &str) -> Option<i64> {
        match HashMap::get(self, key) {
            Some(MetaType::Int64(v)) => Some(*v),
            _ => None,
        }
    }

    fn set(&mut self, key: &str, value: i64) {
        HashMap::insert(self, key.to_string(), MetaType::Int64(value));
    }
}

impl MapLike<f64> for DefaultMeta {
    fn get(&self, key: &str) -> Option<f64> {
        match HashMap::get(self, key) {
            Some(MetaType::Float64(v)) => Some(*v),
            _ => None,
        }
    }

    fn set(&mut self, key: &str, value: f64) {
        HashMap::insert(self, key.to_string(), MetaType::Float64(value));
    }
}

impl MapLike<String> for DefaultMeta {
    fn get(&self, key: &str) -> Option<String> {
        match HashMap::get(self, key) {
            Some(MetaType::String(v)) => Some(v.clone()),
            _ => None,
        }
    }

    fn set(&mut self, key: &str, value: String) {
        HashMap::insert(self, key.to_string(), MetaType::String(value));
    }
}

impl MapLike<bool> for DefaultMeta {
    fn get(&self, key: &str) -> Option<bool> {
        match HashMap::get(self, key) {
            Some(MetaType::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    fn set(&mut self, key: &str, value: bool) {
        HashMap::insert(self, key.to_string(), MetaType::Bool(value));
    }
}

impl MapLike<[[f64; 3]; 3]> for DefaultMeta {
    fn get(&self, _key: &str) -> Option<[[f64; 3]; 3]> {
        // For simplicity, we'll store the box as a special encoding
        // In a real implementation, you might want to add a new MetaType variant
        None
    }

    fn set(&mut self, _key: &str, _value: [[f64; 3]; 3]) {
        // Placeholder implementation
        // In a real implementation, you might want to add MetaType::Array3x3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_type_variants() {
        let int_meta = MetaType::Int64(42);
        let float_meta = MetaType::Float64(3.14);
        let string_meta = MetaType::String("hello".to_string());
        let bool_meta = MetaType::Bool(true);

        match int_meta {
            MetaType::Int64(v) => assert_eq!(v, 42),
            _ => panic!("Expected Int64"),
        }

        match float_meta {
            MetaType::Float64(v) => assert!((v - 3.14).abs() < 1e-10),
            _ => panic!("Expected Float64"),
        }

        match string_meta {
            MetaType::String(v) => assert_eq!(v, "hello"),
            _ => panic!("Expected String"),
        }

        match bool_meta {
            MetaType::Bool(v) => assert!(v),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_default_meta_maplike_i64() {
        let mut meta = DefaultMeta::new();
        
        // Test set and get for i64
        MapLike::<i64>::set(&mut meta, "count", 100i64);
        assert_eq!(MapLike::<i64>::get(&meta, "count"), Some(100i64));
        
        // Test non-existent key
        assert_eq!(MapLike::<i64>::get(&meta, "nonexistent"), None);
        
        // Test overwrite
        MapLike::<i64>::set(&mut meta, "count", 200i64);
        assert_eq!(MapLike::<i64>::get(&meta, "count"), Some(200i64));
    }

    #[test]
    fn test_default_meta_maplike_f64() {
        let mut meta = DefaultMeta::new();
        
        // Test set and get for f64
        MapLike::<f64>::set(&mut meta, "temperature", 298.15);
        let temp: Option<f64> = MapLike::<f64>::get(&meta, "temperature");
        assert!(temp.is_some());
        assert!((temp.unwrap() - 298.15).abs() < 1e-10);
        
        // Test non-existent key
        assert_eq!(MapLike::<f64>::get(&meta, "pressure"), None);
    }

    #[test]
    fn test_default_meta_maplike_string() {
        let mut meta = DefaultMeta::new();
        
        // Test set and get for String
        MapLike::<String>::set(&mut meta, "name", "water".to_string());
        assert_eq!(MapLike::<String>::get(&meta, "name"), Some("water".to_string()));
        
        // Test with different string
        MapLike::<String>::set(&mut meta, "formula", "H2O".to_string());
        assert_eq!(MapLike::<String>::get(&meta, "formula"), Some("H2O".to_string()));
    }

    #[test]
    fn test_default_meta_maplike_bool() {
        let mut meta = DefaultMeta::new();
        
        // Test set and get for bool
        MapLike::<bool>::set(&mut meta, "periodic", true);
        assert_eq!(MapLike::<bool>::get(&meta, "periodic"), Some(true));
        
        MapLike::<bool>::set(&mut meta, "frozen", false);
        assert_eq!(MapLike::<bool>::get(&meta, "frozen"), Some(false));
    }

    #[test]
    fn test_default_meta_type_safety() {
        let mut meta = DefaultMeta::new();
        
        // Set as Int64
        MapLike::<i64>::set(&mut meta, "value", 42i64);
        
        // Try to get as f64 - should return None due to type mismatch
        let as_float: Option<f64> = MapLike::<f64>::get(&meta, "value");
        assert_eq!(as_float, None);
        
        // Get as i64 - should work
        let as_int: Option<i64> = MapLike::<i64>::get(&meta, "value");
        assert_eq!(as_int, Some(42));
    }

    #[test]
    fn test_frame_creation() {
        let frame: Frame = Frame::default();
        assert_eq!(frame.components.len(), 0);
    }

    #[test]
    fn test_frame_with_custom_meta() {
        let mut meta = DefaultMeta::new();
        MapLike::<i64>::set(&mut meta, "timestep", 100i64);
        MapLike::<f64>::set(&mut meta, "temperature", 300.0);
        
        let frame = Frame {
            components: HashMap::new(),
            meta,
        };
        
        let timestep: Option<i64> = MapLike::<i64>::get(&frame.meta, "timestep");
        assert_eq!(timestep, Some(100));
        
        let temp: Option<f64> = MapLike::<f64>::get(&frame.meta, "temperature");
        assert!(temp.is_some());
        assert!((temp.unwrap() - 300.0).abs() < 1e-10);
    }

    #[test]
    fn test_has_timestep_trait() {
        let mut frame: Frame = Frame::default();
        
        // Initially no timestep
        assert_eq!(frame.timestep(), None);
        
        // Set timestep
        frame.set_timestep(1000);
        assert_eq!(frame.timestep(), Some(1000));
        
        // Update timestep
        frame.set_timestep(2000);
        assert_eq!(frame.timestep(), Some(2000));
    }

    #[test]
    fn test_frame_clone() {
        let mut meta = DefaultMeta::new();
        MapLike::<i64>::set(&mut meta, "step", 42i64);
        
        let frame1 = Frame {
            components: HashMap::new(),
            meta,
        };
        
        let frame2 = frame1.clone();
        let step: Option<i64> = MapLike::<i64>::get(&frame2.meta, "step");
        assert_eq!(step, Some(42));
    }

    #[test]
    fn test_multiple_meta_fields() {
        let mut frame: Frame = Frame::default();
        
        frame.set_timestep(100);
        MapLike::<f64>::set(&mut frame.meta, "temperature", 300.0);
        MapLike::<f64>::set(&mut frame.meta, "pressure", 1.0);
        MapLike::<String>::set(&mut frame.meta, "ensemble", "NVT".to_string());
        MapLike::<bool>::set(&mut frame.meta, "periodic", true);
        
        assert_eq!(frame.timestep(), Some(100));
        assert_eq!(MapLike::<f64>::get(&frame.meta, "temperature"), Some(300.0));
        assert_eq!(MapLike::<f64>::get(&frame.meta, "pressure"), Some(1.0));
        assert_eq!(MapLike::<String>::get(&frame.meta, "ensemble"), Some("NVT".to_string()));
        assert_eq!(MapLike::<bool>::get(&frame.meta, "periodic"), Some(true));
    }

    #[test]
    fn test_metatype_debug() {
        let meta = MetaType::Int64(42);
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("Int64"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_metatype_clone() {
        let meta1 = MetaType::String("test".to_string());
        let meta2 = meta1.clone();
        
        match (meta1, meta2) {
            (MetaType::String(s1), MetaType::String(s2)) => {
                assert_eq!(s1, s2);
            },
            _ => panic!("Clone should preserve type"),
        }
    }
}