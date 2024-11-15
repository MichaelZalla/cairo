use crate::{
    animation::lerp,
    color::{self, Color},
    mesh::Face,
    render::{
        culling::{FaceCullingReject, FaceCullingWindingOrder},
        options::RenderPassFlag,
        Renderer,
    },
    software_renderer::SoftwareRenderer,
    vec::vec4::Vec4,
    vertex::default_vertex_out::DefaultVertexOut,
};

pub(in crate::software_renderer) mod clip;

use self::clip::clip_by_all_planes;

#[derive(Default, Debug, Copy, Clone)]
pub struct Triangle<T> {
    pub v0: T,
    pub v1: T,
    pub v2: T,
}

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn process_triangles(
        &mut self,
        faces: &[Face],
        projection_space_vertices: &[DefaultVertexOut],
    ) {
        for face_index in 0..faces.len() {
            // Cull backfaces

            let vertex_index = face_index * 3;

            let mut v0 = projection_space_vertices[vertex_index];
            let mut v1 = projection_space_vertices[vertex_index + 1];
            let mut v2 = projection_space_vertices[vertex_index + 2];

            match self
                .options
                .rasterizer_options
                .face_culling_strategy
                .winding_order
            {
                FaceCullingWindingOrder::Clockwise => {
                    (v0, v1, v2) = (v2, v1, v0);
                }
                FaceCullingWindingOrder::CounterClockwise => {
                    // Use default (counter-clockwise) ordering.
                }
            }

            match self.options.rasterizer_options.face_culling_strategy.reject {
                FaceCullingReject::None => {
                    // Render all faces.
                }
                FaceCullingReject::Backfaces => {
                    // Reject backfaces.

                    if self.is_backface(
                        v0.position_projection_space,
                        v1.position_projection_space,
                        v2.position_projection_space,
                    ) {
                        continue;
                    }
                }
                FaceCullingReject::Frontfaces => {
                    // Reject frontfaces.

                    if !self.is_backface(
                        v0.position_projection_space,
                        v1.position_projection_space,
                        v2.position_projection_space,
                    ) {
                        continue;
                    }
                }
            }

            self.process_triangle(&Triangle { v0, v1, v2 });
        }
    }

    pub(in crate::software_renderer) fn should_cull_from_homogeneous_space(
        &mut self,
        triangle: &Triangle<DefaultVertexOut>,
    ) -> bool {
        let (v0, v1, v2) = (
            &triangle.v0.position_projection_space,
            &triangle.v1.position_projection_space,
            &triangle.v2.position_projection_space,
        );

        if v0.x > v0.w && v1.x > v1.w && v2.x > v2.w {
            return true;
        }

        if v0.x < -v0.w && v1.x < -v1.w && v2.x < -v2.w {
            return true;
        }

        if v0.y > v0.w && v1.y > v1.w && v2.y > v2.w {
            return true;
        }

        if v0.y < -v0.w && v1.y < -v1.w && v2.y < -v2.w {
            return true;
        }

        if v0.z > v0.w && v1.z > v1.w && v2.z > v2.w {
            return true;
        }

        if v0.z < 0.0 && v1.z < 0.0 && v2.z < 0.0 {
            return true;
        }

        false
    }

    fn post_process_triangle_vertices(&mut self, triangle: &Triangle<DefaultVertexOut>) {
        // World-space to screen-space (NDC) transform

        let projection_space_vertices = [triangle.v0, triangle.v1, triangle.v2];

        let mut ndc_space_vertices = projection_space_vertices;

        self.transform_to_ndc_space(&mut ndc_space_vertices[0]);
        self.transform_to_ndc_space(&mut ndc_space_vertices[1]);
        self.transform_to_ndc_space(&mut ndc_space_vertices[2]);

        // Interpolate entire vertex (all attributes) when drawing (scanline
        // interpolant)

        if self
            .options
            .render_pass_flags
            .contains(RenderPassFlag::Rasterization)
        {
            self.triangle_fill(
                ndc_space_vertices[0],
                ndc_space_vertices[1],
                ndc_space_vertices[2],
            );
        }

        if self.options.draw_wireframe {
            let wireframe_color = Color::from_vec3(self.options.wireframe_color * 255.0);

            for i in 0..3 {
                self.render_line(
                    projection_space_vertices[i].position_world_space,
                    projection_space_vertices[if i == 2 { 0 } else { i + 1 }].position_world_space,
                    wireframe_color,
                );
            }
        }

        if self.options.draw_normals {
            for vertex in &projection_space_vertices {
                self.render_line(
                    vertex.position_world_space,
                    vertex.position_world_space
                        + vertex.normal_world_space.to_vec3() * self.options.draw_normals_scale,
                    color::BLUE,
                );

                self.render_line(
                    vertex.position_world_space,
                    vertex.position_world_space
                        + vertex.tangent_world_space.to_vec3() * self.options.draw_normals_scale,
                    color::RED,
                );

                self.render_line(
                    vertex.position_world_space,
                    vertex.position_world_space
                        + vertex.bitangent_world_space.to_vec3() * self.options.draw_normals_scale,
                    color::GREEN,
                );
            }
        }
    }

    fn is_backface(&mut self, v0: Vec4, v1: Vec4, v2: Vec4) -> bool {
        // Computes a hard surface normal for the face (ignores smooth normals);

        let face_normal_unnormalized = (v1 - v0).cross(v2 - v0);

        let similarity_to_view_direction = face_normal_unnormalized.dot(v0);

        similarity_to_view_direction > 0.0
    }

    fn process_triangle(&mut self, triangle: &Triangle<DefaultVertexOut>) {
        // @TODO(mzalla) Geometry shader?

        if self.should_cull_from_homogeneous_space(triangle) {
            return;
        }

        let clipped_triangles = clip_by_all_planes(triangle);

        for clipped in &clipped_triangles {
            self.post_process_triangle_vertices(clipped);
        }
    }

    fn triangle_fill(&mut self, v0: DefaultVertexOut, v1: DefaultVertexOut, v2: DefaultVertexOut) {
        let mut tri = [v0, v1, v2];

        // Sorts points by y-value (highest-to-lowest)

        if tri[1].position_projection_space.y < tri[0].position_projection_space.y {
            tri.swap(0, 1);
        }
        if tri[2].position_projection_space.y < tri[1].position_projection_space.y {
            tri.swap(1, 2);
        }
        if tri[1].position_projection_space.y < tri[0].position_projection_space.y {
            tri.swap(0, 1);
        }

        if tri[0].position_projection_space.y == tri[1].position_projection_space.y {
            // Flat-top (horizontal line is tri[0]-to-tri[1]);

            // tri[2] must sit below tri[0] and tri[1]; tri[0] and tri[1] cannot
            // have the same x-value; therefore, sort tri[0] and tri[1] by x-value;

            if tri[1].position_projection_space.x < tri[0].position_projection_space.x {
                tri.swap(0, 1);
            }

            self.flat_top_triangle_fill(tri[0], tri[1], tri[2]);
        } else if tri[1].position_projection_space.y == tri[2].position_projection_space.y {
            // Flat-bottom (horizontal line is tri[1]-to-tri[2]);

            // tri[0] must sit above tri[1] and tri[2]; tri[1] and tri[2] cannot
            // have the same x-value; therefore, sort tri[1] and tri[2] by x-value;

            if tri[2].position_projection_space.x < tri[1].position_projection_space.x {
                tri.swap(1, 2);
            }

            self.flat_bottom_triangle_fill(tri[0], tri[1], tri[2]);
        } else {
            // Find splitting vertex

            let alpha_split = (tri[1].position_projection_space.y
                - tri[0].position_projection_space.y)
                / (tri[2].position_projection_space.y - tri[0].position_projection_space.y);

            let split_vertex = lerp(tri[0], tri[2], alpha_split);

            if tri[1].position_projection_space.x < split_vertex.position_projection_space.x {
                // Major right

                // tri[0] must sit above tri[1] and split_point; tri[1] and
                // split_point cannot have the same x-value; therefore, sort tri[1]
                // and split_point by x-value;

                self.flat_bottom_triangle_fill(tri[0], tri[1], split_vertex);

                self.flat_top_triangle_fill(tri[1], split_vertex, tri[2]);
            } else {
                // Major left

                self.flat_bottom_triangle_fill(tri[0], split_vertex, tri[1]);

                self.flat_top_triangle_fill(split_vertex, tri[1], tri[2]);
            }
        }
    }

    fn flat_top_triangle_fill(
        &mut self,
        top_left: DefaultVertexOut,
        top_right: DefaultVertexOut,
        bottom: DefaultVertexOut,
    ) {
        let delta_y = bottom.position_projection_space.y - top_left.position_projection_space.y;

        // Calculate the change (step) for left and right sides, as we
        // rasterize downwards with each scanline.
        let top_left_step = (bottom - top_left) / delta_y;
        let top_right_step = (bottom - top_right) / delta_y;

        // Create the right edge interpolant.
        let mut right_edge_interpolant = top_right;

        self.flat_triangle_fill(
            &top_left,
            &bottom,
            &top_left_step,
            &top_right_step,
            &mut right_edge_interpolant,
        );
    }

    fn flat_bottom_triangle_fill(
        &mut self,
        top: DefaultVertexOut,
        bottom_left: DefaultVertexOut,
        bottom_right: DefaultVertexOut,
    ) {
        let delta_y = bottom_right.position_projection_space.y - top.position_projection_space.y;

        // Calculate the change (step) for both left and right sides, as we
        // rasterize downwards with each scanline.
        let bottom_left_step = (bottom_left - top) / delta_y;
        let bottom_right_step = (bottom_right - top) / delta_y;

        // Create the right edge interpolant.
        let mut right_edge_interpolant = top;

        self.flat_triangle_fill(
            &top,
            &bottom_right,
            &bottom_left_step,
            &bottom_right_step,
            &mut right_edge_interpolant,
        );
    }

    fn flat_triangle_fill(
        &mut self,
        it0: &DefaultVertexOut,
        it2: &DefaultVertexOut,
        left_step: &DefaultVertexOut,
        right_step: &DefaultVertexOut,
        right_edge_interpolant: &mut DefaultVertexOut,
    ) {
        // it0 will always be a top vertex.
        // it1 is either a top or a bottom vertex.
        // it2 will always be a bottom vertex.

        // Case 1. Flat-top triangle:
        //  - Left-edge interpolant begins at top-left vertex.
        //  - Right-edge interpolant begins at top-right vertex.

        // Case 2. Flat-bottom triangle:
        //  - Left-edge and right-edge interpolants both begin at top vertex.

        // Left edge is always it0
        let mut left_edge_interpolant = *it0;

        // Calculate our start and end Y (end here is non-inclusive), such that
        // they are non-fractional screen coordinates.
        let y_start: u32 = u32::max((it0.position_projection_space.y - 0.5).ceil() as u32, 0);
        let y_end: u32 = u32::min(
            (it2.position_projection_space.y - 0.5).ceil() as u32,
            self.viewport.height - 1,
        );

        // Adjust both interpolants to account for us snapping y-start and y-end
        // to their nearest whole pixel coordinates.
        left_edge_interpolant +=
            *left_step * (y_start as f32 + 0.5 - it0.position_projection_space.y);
        *right_edge_interpolant +=
            *right_step * (y_start as f32 + 0.5 - it0.position_projection_space.y);

        // Rasterization loop
        for y in y_start..y_end {
            // Calculate our start and end X (end here is non-inclusive), such
            // that they are non-fractional screen coordinates.
            let x_start = u32::max(
                (left_edge_interpolant.position_projection_space.x - 0.5).ceil() as u32,
                0,
            );

            let x_end = u32::min(
                (right_edge_interpolant.position_projection_space.x - 0.5).ceil() as u32,
                self.viewport.width - 1,
            );

            // Create an interpolant that we can move across our horizontal
            // scanline.
            let mut line_interpolant = left_edge_interpolant;

            // Calculate the width of our scanline, for this Y position.
            let dx = right_edge_interpolant.position_projection_space.x
                - left_edge_interpolant.position_projection_space.x;

            // Calculate the change (step) for our horizontal interpolant, based
            // on the width of our scanline.
            let line_interpolant_step = (*right_edge_interpolant - line_interpolant) / dx;

            // Prestep our scanline interpolant to account for us snapping
            // x-start and x-end to their nearest whole pixel coordinates.
            line_interpolant += line_interpolant_step
                * ((x_start as f32) + 0.5 - left_edge_interpolant.position_projection_space.x);

            for x in x_start..x_end {
                self.test_and_set_z_buffer(x, y, &mut line_interpolant);

                line_interpolant += line_interpolant_step;
            }

            left_edge_interpolant += *left_step;
            *right_edge_interpolant += *right_step;
        }
    }
}
