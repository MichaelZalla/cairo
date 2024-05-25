use crate::{matrix::Mat4, mesh::Mesh, pipeline::Pipeline, scene::camera::frustum::Frustum};

impl Pipeline {
    pub(in crate::pipeline) fn _render_entity(
        &mut self,
        world_transform: &Mat4,
        clipping_camera_frustum: &Option<Frustum>,
        entity_mesh: &Mesh,
        entity_material_name: &Option<String>,
    ) -> bool {
        let mut should_cull = false;

        if let Some(frustum) = clipping_camera_frustum.as_ref() {
            if self.should_cull_aabb(*world_transform, frustum, &entity_mesh.aabb) {
                should_cull = true;
            }
        }

        let mut did_set_active_material = false;

        if !should_cull {
            {
                let mut context = self.shader_context.borrow_mut();

                match &entity_material_name {
                    Some(name) => {
                        context.set_active_material(Some(name.clone()));

                        did_set_active_material = true;
                    }
                    None => (),
                }
            }

            self.render_entity_mesh(entity_mesh, world_transform);
        }

        if did_set_active_material {
            // Reset the shader context's original active material.

            let mut context = self.shader_context.borrow_mut();

            context.set_active_material(None);
        }

        !should_cull
    }
}
