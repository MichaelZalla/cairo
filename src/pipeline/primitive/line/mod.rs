use crate::{
    color::{self, Color},
    graphics::Graphics,
    pipeline::Pipeline,
    shader::{
        alpha::AlphaShader, fragment::FragmentShader, geometry::GeometryShader,
        vertex::VertexShader,
    },
    vec::vec3::Vec3,
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

impl<'a, F, V, A, G> Pipeline<'a, F, V, A, G>
where
    F: FragmentShader<'a>,
    V: VertexShader<'a>,
    A: AlphaShader<'a>,
    G: GeometryShader<'a>,
{
    pub fn render_line(&mut self, start: Vec3, end: Vec3, color: Color) {
        let start_vertex_in = DefaultVertexIn {
            p: start,
            c: color.to_vec3() / 255.0,
            ..Default::default()
        };

        let end_vertex_in = DefaultVertexIn {
            p: end,
            c: color.to_vec3() / 255.0,
            ..Default::default()
        };

        let mut start_vertex_out = self.vertex_shader.call(&start_vertex_in);
        let mut end_vertex_out = self.vertex_shader.call(&end_vertex_in);

        self.render_line_from_out_vertices(&mut start_vertex_out, &mut end_vertex_out, color);
    }

    pub fn render_point_indicator(&mut self, position: Vec3, scale: f32) {
        // X-axis (red)

        self.render_line(
            Vec3 {
                x: -1.0 * scale,
                y: 0.0,
                z: 0.0,
            } + position,
            Vec3 {
                x: 1.0 * scale,
                y: 0.0,
                z: 0.0,
            } + position,
            color::RED,
        );

        // Y-axis (blue)

        self.render_line(
            Vec3 {
                x: 0.0,
                y: -1.0 * scale,
                z: 0.0,
            } + position,
            Vec3 {
                x: 0.0,
                y: 1.0 * scale,
                z: 0.0,
            } + position,
            color::BLUE,
        );

        // Z-axis (green)

        self.render_line(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0 * scale,
            } + position,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0 * scale,
            } + position,
            color::GREEN,
        );
    }

    pub fn render_world_axes(&mut self, scale: f32) {
        self.render_point_indicator(Default::default(), scale)
    }

    pub fn render_ground_plane(&mut self, scale: f32) {
        for i in -10..10 + 1 {
            // X-axis parallels

            self.render_line(
                Vec3 {
                    x: -10.0 * scale,
                    z: (i as f32 * scale),
                    ..Default::default()
                },
                Vec3 {
                    x: 10.0 * scale,
                    z: (i as f32 * scale),
                    ..Default::default()
                },
                if i == 0 { color::RED } else { color::WHITE },
            );

            // Z-axis parallels

            self.render_line(
                Vec3 {
                    x: (i as f32 * scale),
                    z: -10.0 * scale,
                    ..Default::default()
                },
                Vec3 {
                    x: (i as f32 * scale),
                    z: 10.0 * scale,
                    ..Default::default()
                },
                if i == 0 { color::GREEN } else { color::WHITE },
            );

            // Y-axis

            self.render_line(
                Vec3 {
                    y: -10.0 * scale,
                    ..Default::default()
                },
                Vec3 {
                    y: 10.0 * scale,
                    ..Default::default()
                },
                color::BLUE,
            );
        }
    }

    fn render_line_from_out_vertices(
        &mut self,
        start: &mut DefaultVertexOut,
        end: &mut DefaultVertexOut,
        color: Color,
    ) {
        self.transform_to_ndc_space(start);
        self.transform_to_ndc_space(end);

        // Cull lines that are completely in front of our near plane (z1 <= 0 and z2 <= 0).
        if start.p.z <= 0.0 && end.p.z <= 0.0 {
            return;
        }

        Graphics::line(
            &mut self.forward_framebuffer.as_mut().unwrap(),
            start.p.x as i32,
            start.p.y as i32,
            end.p.x as i32,
            end.p.y as i32,
            color,
        );
    }
}
