use crate::{
    geometry::primitives::aabb::AABB, matrix::Mat4, mesh::Mesh, resource::handle::Handle,
    scene::camera::frustum::Frustum, software_renderer::SoftwareRenderer, vec::vec4::Vec4,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_entity(
        &mut self,
        world_transform: &Mat4,
        clipping_camera_frustum: &Option<Frustum>,
        entity_mesh: &Mesh,
        entity_material: &Option<Handle>,
    ) -> bool {
        let mut should_cull = false;

        if let Some(frustum) = clipping_camera_frustum.as_ref() {
            if should_cull_aabb(*world_transform, frustum, &entity_mesh.aabb) {
                should_cull = true;
            }
        }

        if !should_cull {
            let mut did_set_active_material = false;

            {
                let mut context = self.shader_context.borrow_mut();

                match &entity_material {
                    Some(handle) => {
                        context.set_active_material(Some(*handle));

                        did_set_active_material = true;
                    }
                    None => (),
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

fn should_cull_aabb(world_transform: Mat4, clipping_camera_frustum: &Frustum, aabb: &AABB) -> bool {
    // Cull the entire entity, if possible, based on its bounds.

    let bounding_sphere_position = (Vec4::new(aabb.center, 1.0) * world_transform).to_vec3();

    // @TODO Generate planes once per frame, not once per entity draw.
    let culling_planes = clipping_camera_frustum.get_planes();

    let radius = aabb.bounding_sphere_radius;

    // @TODO Verify the sign of the top plane and bottom plane normals.
    !culling_planes[0].is_on_or_in_front_of(&bounding_sphere_position, radius)
        || !culling_planes[1].is_on_or_in_front_of(&bounding_sphere_position, radius)
        || !culling_planes[2].is_on_or_in_front_of(&bounding_sphere_position, radius)
        || !culling_planes[3].is_on_or_in_front_of(&bounding_sphere_position, radius)
        || !culling_planes[4].is_on_or_in_front_of(&bounding_sphere_position, radius)
        || !culling_planes[5].is_on_or_in_front_of(&bounding_sphere_position, radius)
}
