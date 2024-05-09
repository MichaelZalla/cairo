use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{app::context::ApplicationRenderingContext, serde::PostDeserialize};

use super::{graph::SceneGraph, resources::SceneResources};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneContext {
    pub resources: Rc<RefCell<SceneResources>>,
    pub scenes: RefCell<Vec<SceneGraph>>,
}

impl Default for SceneContext {
    fn default() -> Self {
        Self {
            resources: Default::default(),
            scenes: RefCell::new(vec![Default::default()]),
        }
    }
}

impl PostDeserialize for SceneContext {
    fn post_deserialize(&mut self) {
        self.resources.borrow_mut().post_deserialize();

        for scene in self.scenes.borrow_mut().iter_mut() {
            scene.post_deserialize();
        }
    }
}

impl SceneContext {
    pub fn load_all_resources(
        &mut self,
        rendering_context: &ApplicationRenderingContext,
    ) -> Result<(), String> {
        // Loads all texture map data.

        let resources = self.resources.borrow_mut();

        let mut textures = resources.texture_u8.borrow_mut();

        // Load all texture map data from materials.

        let mut materials = resources.material.borrow_mut();

        for material in materials.values_mut() {
            let arena = &mut *textures;

            material.load_all_maps(arena, rendering_context)?;
        }

        // Load all texture map data from cubemaps.

        let mut cubemaps = resources.cubemap_u8.borrow_mut();

        for slot in cubemaps.entries.iter_mut() {
            match slot {
                Some(entry) => {
                    let cubemap = &mut entry.item;

                    cubemap.load(rendering_context)?;
                }
                None => (),
            }
        }

        Ok(())
    }
}
