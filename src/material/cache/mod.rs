use std::collections::HashMap;

use super::Material;

#[derive(Debug, Default)]
pub struct MaterialCache {
    map: HashMap<String, Material>,
}

impl MaterialCache {
    pub fn get(&self, key: &String) -> Option<&Material> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &String) -> Option<&mut Material> {
        self.map.get_mut(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, String, Material> {
        self.map.values()
    }

    pub fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, String, Material> {
        self.map.values_mut()
    }

    pub fn insert(&mut self, new_entry: Material) {
        let key = new_entry.name.clone();
        let _old_entry = self.map.insert(key, new_entry);
    }
}
