use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{app::context::ApplicationRenderingContext, serde::PostDeserialize};

use super::{graph::SceneGraph, resources::SceneResources};

pub mod utils;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SceneContext {
    pub resources: Rc<SceneResources>,
    pub scenes: RefCell<Vec<SceneGraph>>,
}

impl PostDeserialize for SceneContext {
    fn post_deserialize(&mut self) {
        self.resources.post_deserialize_non_mut();

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

        let mut texture_u8_arena = self.resources.texture_u8.borrow_mut();

        // Load all texture map data from materials.

        let mut material_arena = self.resources.material.borrow_mut();

        for entry in material_arena.entries.iter_mut().flatten() {
            let material = &mut entry.item;

            let texture_arena = &mut *texture_u8_arena;

            material.load_all_maps(texture_arena, rendering_context)?;
        }

        // Load all texture map data from cubemaps.

        let mut cubemap_u8_arena = self.resources.cubemap_u8.borrow_mut();

        for slot in cubemap_u8_arena.entries.iter_mut() {
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
