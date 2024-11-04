use crate::{
    color::Color,
    material::Material,
    mesh,
    resource::{arena::Arena, handle::Handle},
    scene::camera::Camera,
    software_renderer::SoftwareRenderer,
    transform::Transform3D,
    vec::vec3::Vec3,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_point(
        &mut self,
        point_world_space: Vec3,
        color: Color,
        camera: Option<&Camera>,
        materials: Option<&mut Arena<Material>>,
        material: Option<Handle>,
        scale: Option<f32>,
    ) {
        let point_ndc_space: Vec3;

        {
            let shader_context = (*self.shader_context).borrow();

            point_ndc_space = shader_context.to_ndc_space(point_world_space);
        }

        let x = (point_ndc_space.x * self.viewport.width as f32) as u32;
        let y = (point_ndc_space.y * self.viewport.height as f32) as u32;
        let z = point_ndc_space.z;

        // Cull points that are in front of our near plane (z <= 0).
        if z <= 0.0 {
            return;
        }

        let color_u32 = color.to_u32();

        if let Some(materials) = materials {
            let material_handle = material.unwrap();

            let billboard_scale = scale.unwrap();

            let mut billboard_mesh = mesh::primitive::billboard::generate(
                point_world_space,
                &camera.unwrap().look_vector.get_position(),
                billboard_scale,
                billboard_scale,
            );

            if let Ok(billboard_material_entry) = materials.get_mut(&material_handle) {
                let material = &mut billboard_material_entry.item;

                material.albedo = color.to_vec3() / 255.0;

                billboard_mesh.material = Some(material_handle);

                let transform: Transform3D = Default::default();

                self.render_entity_mesh(&billboard_mesh, transform.mat());

                return;
            }
        }

        if let Some(framebuffer_rc) = &self.framebuffer {
            let framebuffer = framebuffer_rc.borrow_mut();

            if let Some(forward_buffer_rc) = &framebuffer.attachments.forward_ldr {
                let mut forward_buffer = forward_buffer_rc.borrow_mut();

                forward_buffer.set(x, y, color_u32);
            }
        }
    }
}
