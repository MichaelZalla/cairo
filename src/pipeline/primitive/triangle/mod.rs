use crate::{
    color,
    mesh::Face,
    pipeline::{
        options::{PipelineFaceCullingReject, PipelineFaceCullingWindingOrder},
        Pipeline,
    },
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
    vertex::default_vertex_out::DefaultVertexOut,
};

#[derive(Default, Debug, Copy, Clone)]
pub struct Triangle<T> {
    pub v0: T,
    pub v1: T,
    pub v2: T,
}

impl<'a> Pipeline<'a> {
    pub(in crate::pipeline) fn process_triangles(
        &mut self,
        faces: &Vec<Face>,
        projection_space_vertices: Vec<DefaultVertexOut>,
    ) {
        let mut triangles: Vec<Triangle<DefaultVertexOut>> = vec![];

        for face_index in 0..faces.len() {
            // Cull backfaces

            let mut v0 = projection_space_vertices[face_index * 3];
            let mut v1 = projection_space_vertices[face_index * 3 + 1];
            let mut v2 = projection_space_vertices[face_index * 3 + 2];

            match self.options.face_culling_strategy.window_order {
                PipelineFaceCullingWindingOrder::Clockwise => {
                    (v0, v1, v2) = (v2, v1, v0);
                }
                PipelineFaceCullingWindingOrder::CounterClockwise => {
                    // Use default (counter-clockwise) ordering.
                }
            }

            match self.options.face_culling_strategy.reject {
                PipelineFaceCullingReject::None => {
                    // Render all faces.
                }
                PipelineFaceCullingReject::Backfaces => {
                    // Reject backfaces.

                    if self.is_backface(v0.position, v1.position, v2.position) {
                        continue;
                    }
                }
                PipelineFaceCullingReject::Frontfaces => {
                    // Reject frontfaces.

                    if !self.is_backface(v0.position, v1.position, v2.position) {
                        continue;
                    }
                }
            }

            triangles.push(Triangle { v0, v1, v2 });
        }

        for triangle in triangles.as_mut_slice() {
            self.process_triangle(triangle);
        }
    }

    pub(in crate::pipeline) fn should_cull_from_homogeneous_space(
        &mut self,
        triangle: &mut Triangle<DefaultVertexOut>,
    ) -> bool {
        if triangle.v0.position.x > triangle.v0.position.w
            && triangle.v1.position.x > triangle.v1.position.w
            && triangle.v2.position.x > triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.x < -triangle.v0.position.w
            && triangle.v1.position.x < -triangle.v1.position.w
            && triangle.v2.position.x < -triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.y > triangle.v0.position.w
            && triangle.v1.position.y > triangle.v1.position.w
            && triangle.v2.position.y > triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.y < -triangle.v0.position.w
            && triangle.v1.position.y < -triangle.v1.position.w
            && triangle.v2.position.y < -triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.z > triangle.v0.position.w
            && triangle.v1.position.z > triangle.v1.position.w
            && triangle.v2.position.z > triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.z < 0.0
            && triangle.v1.position.z < 0.0
            && triangle.v2.position.z < 0.0
        {
            return true;
        }

        return false;
    }

    fn post_process_triangle_vertices(&mut self, triangle: &mut Triangle<DefaultVertexOut>) {
        // World-space to screen-space (NDC) transform

        let projection_space_vertices = [triangle.v0, triangle.v1, triangle.v2];

        let mut ndc_space_vertices = projection_space_vertices.clone();

        self.transform_to_ndc_space(&mut ndc_space_vertices[0]);
        self.transform_to_ndc_space(&mut ndc_space_vertices[1]);
        self.transform_to_ndc_space(&mut ndc_space_vertices[2]);

        // Interpolate entire vertex (all attributes) when drawing (scanline
        // interpolant)

        if self.options.do_rasterized_geometry {
            self.triangle_fill(
                ndc_space_vertices[0],
                ndc_space_vertices[1],
                ndc_space_vertices[2],
            );
        }

        if self.options.do_wireframe {
            for i in 0..3 {
                self.render_line(
                    projection_space_vertices[i].world_pos,
                    projection_space_vertices[if i == 2 { 0 } else { i + 1 }].world_pos,
                    color::WHITE,
                );
            }
        }

        if self.options.do_visualize_normals {
            for i in 0..3 {
                self.render_line(
                    projection_space_vertices[i].world_pos,
                    projection_space_vertices[i].world_pos
                        + projection_space_vertices[i].normal.to_vec3(),
                    color::BLUE,
                );

                let tangent_world_space = Vec4::new(vec3::LEFT * -1.0, 1.0)
                    * projection_space_vertices[i].tangent_space_info.tbn;

                self.render_line(
                    projection_space_vertices[i].world_pos,
                    projection_space_vertices[i].world_pos + tangent_world_space.to_vec3(),
                    color::RED,
                );

                let bitangent_world_space = Vec4::new(vec3::UP * -1.0, 1.0)
                    * projection_space_vertices[i].tangent_space_info.tbn;

                self.render_line(
                    projection_space_vertices[i].world_pos,
                    projection_space_vertices[i].world_pos + bitangent_world_space.to_vec3(),
                    color::GREEN,
                );
            }
        }
    }

    fn is_backface(&mut self, v0: Vec4, v1: Vec4, v2: Vec4) -> bool {
        let vertices = [
            Vec3 {
                x: v0.x,
                y: v0.y,
                z: v0.z,
            },
            Vec3 {
                x: v1.x,
                y: v1.y,
                z: v1.z,
            },
            Vec3 {
                x: v2.x,
                y: v2.y,
                z: v2.z,
            },
        ];

        // Computes a hard surface normal for the face (ignores smooth normals);

        let vertex_normal = (vertices[1] - vertices[0])
            .cross(vertices[2] - vertices[0])
            .as_normal();

        let projected_origin = Vec4::new(Default::default(), 1.0)
            * self.shader_context.read().unwrap().get_projection();

        let dot_product = vertex_normal.dot(
            vertices[0].as_normal()
                - Vec3 {
                    x: projected_origin.x,
                    y: projected_origin.y,
                    z: projected_origin.z,
                },
        );

        if dot_product > 0.0 {
            return true;
        }

        return false;
    }

    fn clip1(&mut self, v0: DefaultVertexOut, v1: DefaultVertexOut, v2: DefaultVertexOut) {
        let a_alpha = -(v0.position.z) / (v1.position.z - v0.position.z);
        let b_alpha = -(v0.position.z) / (v2.position.z - v0.position.z);

        let a_prime = DefaultVertexOut::interpolate(v0, v1, a_alpha);
        let b_prime = DefaultVertexOut::interpolate(v0, v2, b_alpha);

        let mut triangle1 = Triangle {
            v0: a_prime,
            v1,
            v2,
        };

        let mut triangle2 = Triangle {
            v0: b_prime,
            v1: a_prime,
            v2,
        };

        self.post_process_triangle_vertices(&mut triangle1);
        self.post_process_triangle_vertices(&mut triangle2);
    }

    fn clip2(&mut self, v0: DefaultVertexOut, v1: DefaultVertexOut, v2: DefaultVertexOut) {
        let a_alpha = -(v0.position.z) / (v2.position.z - v0.position.z);
        let b_alpha = -(v1.position.z) / (v2.position.z - v1.position.z);

        let a_prime = DefaultVertexOut::interpolate(v0, v2, a_alpha);
        let b_prime = DefaultVertexOut::interpolate(v1, v2, b_alpha);

        let mut triangle = Triangle {
            v0: a_prime,
            v1: b_prime,
            v2,
        };

        self.post_process_triangle_vertices(&mut triangle);
    }

    fn process_triangle(&mut self, triangle: &mut Triangle<DefaultVertexOut>) {
        // @TODO(mzalla) Geometry shader?

        if self.should_cull_from_homogeneous_space(triangle) {
            return;
        }

        // Clip triangles that intersect the front of our view frustum

        if triangle.v0.position.z < 0.0 {
            if triangle.v1.position.z < 0.0 {
                // Clip 2 (0 and 1)
                self.clip2(triangle.v0, triangle.v1, triangle.v2);
            } else if triangle.v2.position.z < 0.0 {
                // Clip 2 (0 and 2)
                self.clip1(triangle.v0, triangle.v2, triangle.v1);
            } else {
                // Clip 1 (0)
                self.clip1(triangle.v0, triangle.v1, triangle.v2);
            }
        } else if triangle.v1.position.z < 0.0 {
            if triangle.v2.position.z < 0.0 {
                // Clip 2
                self.clip2(triangle.v1, triangle.v2, triangle.v0);
            } else {
                // Clip 1
                self.clip1(triangle.v1, triangle.v0, triangle.v2);
            }
        } else if triangle.v2.position.z < 0.0 {
            // Clip 1
            self.clip1(triangle.v2, triangle.v0, triangle.v1);
        } else {
            self.post_process_triangle_vertices(triangle);
        }
    }

    fn triangle_fill(&mut self, v0: DefaultVertexOut, v1: DefaultVertexOut, v2: DefaultVertexOut) {
        let mut tri = vec![v0, v1, v2];

        // Sorts points by y-value (highest-to-lowest)

        if tri[1].position.y < tri[0].position.y {
            tri.swap(0, 1);
        }
        if tri[2].position.y < tri[1].position.y {
            tri.swap(1, 2);
        }
        if tri[1].position.y < tri[0].position.y {
            tri.swap(0, 1);
        }

        if tri[0].position.y == tri[1].position.y {
            // Flat-top (horizontal line is tri[0]-to-tri[1]);

            // tri[2] must sit below tri[0] and tri[1]; tri[0] and tri[1] cannot
            // have the same x-value; therefore, sort tri[0] and tri[1] by x-value;

            if tri[1].position.x < tri[0].position.x {
                tri.swap(0, 1);
            }

            self.flat_top_triangle_fill(tri[0], tri[1], tri[2]);
        } else if tri[1].position.y == tri[2].position.y {
            // Flat-bottom (horizontal line is tri[1]-to-tri[2]);

            // tri[0] must sit above tri[1] and tri[2]; tri[1] and tri[2] cannot
            // have the same x-value; therefore, sort tri[1] and tri[2] by x-value;

            if tri[2].position.x < tri[1].position.x {
                tri.swap(1, 2);
            }

            self.flat_bottom_triangle_fill(tri[0], tri[1], tri[2]);
        } else {
            // Find splitting vertex

            let alpha_split =
                (tri[1].position.y - tri[0].position.y) / (tri[2].position.y - tri[0].position.y);

            let split_vertex = DefaultVertexOut::interpolate(tri[0], tri[2], alpha_split);

            if tri[1].position.x < split_vertex.position.x {
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
        let delta_y = bottom.position.y - top_left.position.y;

        // Calculate the change (step) for left and right sides, as we
        // rasterize downwards with each scanline.
        let top_left_step = (bottom - top_left) / delta_y;
        let top_right_step = (bottom - top_right) / delta_y;

        // Create the right edge interpolant.
        let mut right_edge_interpolant = top_right;

        self.flat_triangle_fill(
            &top_left,
            // &top_right,
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
        let delta_y = bottom_right.position.y - top.position.y;

        // Calculate the change (step) for both left and right sides, as we
        // rasterize downwards with each scanline.
        let bottom_left_step = (bottom_left - top) / delta_y;
        let bottom_right_step = (bottom_right - top) / delta_y;

        // Create the right edge interpolant.
        let mut right_edge_interpolant = top;

        self.flat_triangle_fill(
            &top,
            // &bottom_left,
            &bottom_right,
            &bottom_left_step,
            &bottom_right_step,
            &mut right_edge_interpolant,
        );
    }

    fn flat_triangle_fill(
        &mut self,
        it0: &DefaultVertexOut,
        // it1: &DefaultVertexOut,
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
        let mut left_edge_interpolant = it0.clone();

        // Calculate our start and end Y (end here is non-inclusive), such that
        // they are non-fractional screen coordinates.
        let y_start: u32 = u32::max((it0.position.y - 0.5).ceil() as u32, 0);
        let y_end: u32 = u32::min(
            (it2.position.y - 0.5).ceil() as u32,
            self.viewport.height - 1,
        );

        // Adjust both interpolants to account for us snapping y-start and y-end
        // to their nearest whole pixel coordinates.
        left_edge_interpolant += *left_step * (y_start as f32 + 0.5 - it0.position.y);
        *right_edge_interpolant += *right_step * (y_start as f32 + 0.5 - it0.position.y);

        // Rasterization loop
        for y in y_start..y_end {
            // Calculate our start and end X (end here is non-inclusive), such
            // that they are non-fractional screen coordinates.
            let x_start = u32::max((left_edge_interpolant.position.x - 0.5).ceil() as u32, 0);
            let x_end = u32::min(
                (right_edge_interpolant.position.x - 0.5).ceil() as u32,
                self.viewport.width - 1,
            );

            // Create an interpolant that we can move across our horizontal
            // scanline.
            let mut line_interpolant = left_edge_interpolant.clone();

            // Calculate the width of our scanline, for this Y position.
            let dx = right_edge_interpolant.position.x - left_edge_interpolant.position.x;

            // Calculate the change (step) for our horizontal interpolant, based
            // on the width of our scanline.
            let line_interpolant_step = (*right_edge_interpolant - line_interpolant) / dx;

            // Prestep our scanline interpolant to account for us snapping
            // x-start and x-end to their nearest whole pixel coordinates.
            line_interpolant +=
                line_interpolant_step * ((x_start as f32) + 0.5 - left_edge_interpolant.position.x);

            for x in x_start..x_end {
                self.test_and_set_z_buffer(x, y, &mut line_interpolant);

                line_interpolant += line_interpolant_step;
            }

            left_edge_interpolant += *left_step;
            *right_edge_interpolant += *right_step;
        }
    }
}