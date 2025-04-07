use crate::{
    matrix::Mat4, mesh::Mesh, resource::handle::Handle, software_renderer::SoftwareRenderer,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_entity(
        &mut self,
        world_transform: &Mat4,
        entity_mesh: &Mesh,
        entity_material: &Option<Handle>,
    ) -> bool {
        let should_cull = self
            .clipping_frustum
            .should_cull_aabb(world_transform, &entity_mesh.aabb);

        if !should_cull {
            let mut did_set_active_material = false;

            {
                let mut context = self.shader_context.borrow_mut();

                if let Some(handle) = &entity_material {
                    context.set_active_material(Some(*handle));

                    did_set_active_material = true;
                }
            }

            self.render_entity_mesh(entity_mesh, world_transform);

            if did_set_active_material {
                // Reset the shader context's original active material.

                let mut context = self.shader_context.borrow_mut();

                context.set_active_material(None);
            }
        }

        !should_cull
    }
}
