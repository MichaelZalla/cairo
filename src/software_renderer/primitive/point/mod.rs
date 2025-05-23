use crate::{
    color::Color,
    matrix::Mat4,
    mesh::Mesh,
    render::{options::RenderPassFlag, Renderer},
    resource::handle::Handle,
    software_renderer::SoftwareRenderer,
    vec::vec4::Vec4,
    vertex::default_vertex_in::DefaultVertexIn,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_point(
        &mut self,
        transform: &Mat4,
        color: Option<Color>,
        mesh: Option<&Mesh>,
        material_handle: Option<Handle>,
    ) {
        match mesh {
            Some(mesh) => {
                let original_material_handle = {
                    let mut shader_context = self.shader_context.borrow_mut();

                    let handle = shader_context.active_material;

                    if let Some(handle) = material_handle {
                        shader_context.active_material.replace(handle);
                    };

                    handle
                };

                self.render_entity(transform, mesh, &material_handle);

                {
                    let mut shader_context = self.shader_context.borrow_mut();

                    shader_context.active_material = original_material_handle;
                }
            }
            None => {
                self.render_point_without_mesh(transform, color.unwrap());
            }
        }
    }

    fn render_point_without_mesh(&mut self, transform: &Mat4, color: Color) {
        // Cull point masses against the culling frustum.

        let position_world_space = (Vec4::position(Default::default()) * *transform).to_vec3();

        for plane in self.clipping_frustum.get_planes() {
            if !plane.is_on_or_in_front_of(&position_world_space, 0.0) {
                return;
            }
        }

        let original_flags = self.options.render_pass_flags;

        self.options.render_pass_flags ^= RenderPassFlag::Lighting;
        self.options.render_pass_flags ^= RenderPassFlag::DeferredLighting;

        let mut color_vec3 = color.to_vec3() / 255.0;

        color_vec3.srgb_to_linear();

        let vertex_out = {
            let shader_context = self.shader_context.borrow();

            let vertex_in = DefaultVertexIn {
                position: position_world_space,
                color: color_vec3,
                ..Default::default()
            };

            (self.vertex_shader)(&shader_context, &vertex_in)
        };

        let projection_space_vertex = vertex_out;

        let mut ndc_space_vertex = projection_space_vertex;

        let v = &ndc_space_vertex.position_projection_space;

        if v.x > v.w || v.x < -v.w {
            return;
        }

        if v.y > v.w || v.y < -v.w {
            return;
        }

        if v.z > v.w || v.z < -v.w {
            return;
        }

        ndc_space_vertex.projection_space_to_viewport_space(&self.viewport);

        let x = u32::max(
            (ndc_space_vertex.position_projection_space.x - 0.5).ceil() as u32,
            0,
        );

        let y = u32::max(
            (ndc_space_vertex.position_projection_space.y - 0.5).ceil() as u32,
            0,
        );

        if x > self.viewport.width - 1 || y > self.viewport.height - 1 {
            return;
        }

        self.submit_fragment(x, y, &mut ndc_space_vertex);

        self.options.render_pass_flags = original_flags;
    }
}
