use std::f32::consts::TAU;

use rand_distr::{Distribution, Uniform};

use crate::{
    animation::{lerp, smooth_step},
    buffer::{framebuffer::StencilBuffer, Buffer2D},
    matrix::Mat4,
    render::options::RenderPassFlag,
    software_renderer::{gbuffer::GBuffer, SoftwareRenderer},
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

pub(in crate::software_renderer) const KERNEL_SIZE: usize = 16;

pub(in crate::software_renderer) fn make_hemisphere_kernel() -> [Vec3; KERNEL_SIZE] {
    let mut rng = rand::thread_rng();

    let uniform = Uniform::<f32>::new(0.0, 1.0);

    let mut hemisphere_kernel: [Vec3; KERNEL_SIZE] = [Default::default(); KERNEL_SIZE];

    for (index, kernel) in hemisphere_kernel.iter_mut().enumerate() {
        let mut scale = index as f32 / KERNEL_SIZE as f32;

        scale = lerp(0.1, 1.0, scale * scale);

        let half_box_sample = Vec3 {
            x: uniform.sample(&mut rng) * 2.0 - 1.0,
            y: uniform.sample(&mut rng) * 2.0 - 1.0,
            z: uniform.sample(&mut rng), // Forward, in tangent space.
        };

        let hemisphere_sample = half_box_sample.as_normal() * scale;

        *kernel = hemisphere_sample;
    }

    hemisphere_kernel
}

pub(in crate::software_renderer) fn make_4x4_tangent_space_rotations() -> [Quaternion; 16] {
    let mut rng = rand::thread_rng();

    let uniform = Uniform::<f32>::new(0.0, 1.0);

    let mut rotational_noise_samples = [Default::default(); 16];

    for sample in rotational_noise_samples.iter_mut() {
        let theta = uniform.sample(&mut rng) * TAU;

        *sample = Quaternion::new(vec3::FORWARD, theta);
    }

    rotational_noise_samples
}

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn do_ssao_pass(&mut self) {
        if let (Some(g_buffer), Some(ssao_buffer), Some(framebuffer_rc)) =
            (&mut self.g_buffer, &mut self.ssao_buffer, &self.framebuffer)
        {
            let occlusion_buffer = &mut ssao_buffer.levels[0].0;

            let framebuffer = &framebuffer_rc.borrow();

            let depth_buffer_rc = framebuffer.attachments.depth.as_ref().unwrap();
            let stencil_buffer_rc = framebuffer.attachments.stencil.as_ref().unwrap();

            let depth_buffer = depth_buffer_rc.borrow();
            let stencil_buffer = stencil_buffer_rc.borrow();

            let (near, far) = (
                depth_buffer.get_projection_z_near(),
                depth_buffer.get_projection_z_far(),
            );

            // Clear our buffer.

            occlusion_buffer.clear(None);

            // 1. Compute approximate screen-space AO for each sample in the
            // G-buffer. Write the approximations to the occlusion buffer.

            match (
                self.ssao_hemisphere_kernel.as_ref(),
                self.ssao_4x4_tangent_space_rotations.as_ref(),
            ) {
                (Some(hemisphere_kernel), Some(tangent_space_rotations)) => {
                    let shader_context = self.shader_context.borrow();

                    for y in 0..g_buffer.0.height {
                        for x in 0..g_buffer.0.width {
                            let geometry_sample = g_buffer.0.get(x, y);
                            let stencil = stencil_buffer.0.get(x, y);

                            if *stencil == 0 {
                                continue;
                            }

                            let normal_view_space = (geometry_sample.normal_world_space
                                * shader_context.view_inverse_transform)
                                .as_normal();

                            let tbn = get_tbn_for_kernel_sample(
                                x,
                                y,
                                normal_view_space,
                                tangent_space_rotations,
                            );

                            let position_view_space = geometry_sample.position_view_space;

                            let occlusion = get_occlusion(
                                near,
                                far,
                                g_buffer,
                                shader_context.get_projection(),
                                hemisphere_kernel,
                                position_view_space,
                                tbn,
                            );

                            occlusion_buffer.set(x, y, occlusion);
                        }
                    }
                }
                _ => panic!(),
            }

            // 2. (Optional) Blur the occlusion buffer to reduce noise
            //    artifacts.

            if self
                .options
                .render_pass_flags
                .contains(RenderPassFlag::SsaoBlur)
            {
                if let Some(ssao_blur_buffer) = self.ssao_blur_buffer.as_mut() {
                    ssao_blur(
                        &stencil_buffer,
                        occlusion_buffer,
                        &mut ssao_blur_buffer.levels[0].0,
                    );
                }
            }

            // 3. Write the final occlusion factors back to the geometry buffer.

            write_ambient_occlusion_factors(occlusion_buffer, g_buffer);
        }
    }
}

fn write_ambient_occlusion_factors(occlusion_map: &Buffer2D<f32>, g_buffer: &mut GBuffer) {
    for (index, occlusion) in occlusion_map.get_all().iter().enumerate() {
        let sample = g_buffer.0.get_at_mut(index);

        sample.ambient_factor = 1.0 - *occlusion;
    }
}

fn get_tbn_for_kernel_sample(
    x: u32,
    y: u32,
    normal: Vec3,
    tangent_space_rotations: &[Quaternion; 16],
) -> Mat4 {
    let (normal, tangent, bitangent) = normal.basis();

    let rotation = {
        let index = {
            let column_index = x.rem_euclid(4) as usize;
            let row_index = y.rem_euclid(4) as usize;

            row_index * 4 + column_index
        };

        tangent_space_rotations[index]
    };

    // Renormalizes after rotation.
    let tangent_rotated = (tangent * *rotation.mat()).as_normal();

    // Renormalizes after rotation.
    let bitangent_rotated = (bitangent * *rotation.mat()).as_normal();

    Mat4::tbn(tangent_rotated, bitangent_rotated, normal)
}

fn transform_to_ndc_space(position_projection_space: Vec4) -> Vec3 {
    let w_inverse = 1.0 / position_projection_space.w;

    let mut position_ndc_space = position_projection_space.to_vec3();

    position_ndc_space *= w_inverse;

    position_ndc_space
}

fn get_occlusion(
    near: f32,
    far: f32,
    geometry_buffer: &GBuffer,
    projection: Mat4,
    hemisphere_kernel: &[Vec3; KERNEL_SIZE],
    position_view_space: Vec3,
    tbn: Mat4,
) -> f32 {
    let mut occlusion = 0.0;

    static KERNEL_RADIUS: f32 = 1.0;

    for sample in hemisphere_kernel {
        let sample_view_space = *sample * tbn;

        let sample_position_view_space = position_view_space + sample_view_space * KERNEL_RADIUS;

        let sample_position_projection_space =
            Vec4::position(sample_position_view_space) * projection;

        let sample_position_ndc_space = transform_to_ndc_space(sample_position_projection_space);

        let sample_position_uv = sample_position_ndc_space.ndc_to_uv();

        let sample_depth_projection_space = sample_position_projection_space.z;

        let closest_depth_projection_space = {
            let x = (sample_position_uv.x * (geometry_buffer.0.width - 1) as f32) as u32;
            let y = ((1.0 - sample_position_uv.y) * (geometry_buffer.0.height - 1) as f32) as u32;

            let index = (y * geometry_buffer.0.width + x) as usize;

            if index < geometry_buffer.0.data.len() {
                let alpha = geometry_buffer.0.get_at(index).depth;

                near + (far - near) * alpha
            } else {
                1.0
            }
        };

        static BIAS: f32 = 0.025;

        occlusion += if closest_depth_projection_space <= sample_depth_projection_space + BIAS {
            let range_check = {
                let depth_delta = closest_depth_projection_space - sample_depth_projection_space;

                smooth_step(0.0, 1.0, KERNEL_RADIUS / depth_delta.abs())
            };

            1.0 * range_check
        } else {
            0.0
        };
    }

    occlusion /= KERNEL_SIZE as f32;

    occlusion
}

fn ssao_blur(
    stencil_buffer: &StencilBuffer,
    ssao_buffer: &mut Buffer2D<f32>,
    ssao_blur_buffer: &mut Buffer2D<f32>,
) {
    static BLUR_STRENGTH: f32 = 2.0;

    let max_offset = BLUR_STRENGTH as i32;

    for y in 0..ssao_buffer.height {
        for x in 0..ssao_buffer.width {
            let stencil = stencil_buffer.0.get(x, y);

            if *stencil == 0 {
                continue;
            }

            let mut accum = 0.0;
            let mut num_contributions = 0;

            for y_offset in -max_offset..max_offset {
                for x_offset in -max_offset..max_offset {
                    let neighbor_x = x as i32 + x_offset;
                    let neighbor_y = y as i32 + y_offset;

                    // Ignores any out-of-bounds neighbors.

                    if neighbor_x < 0
                        || neighbor_x >= ssao_buffer.width as i32
                        || neighbor_y < 0
                        || neighbor_y >= ssao_buffer.height as i32
                    {
                        continue;
                    }

                    // Don't contribute occlusion for pixels that weren't
                    // rasterized this frame.

                    let neighbor_stencil =
                        stencil_buffer.0.get(neighbor_x as u32, neighbor_y as u32);

                    if *neighbor_stencil == 0 {
                        continue;
                    }

                    let neighbor_occlusion = *ssao_buffer.get(neighbor_x as u32, neighbor_y as u32);

                    accum += neighbor_occlusion;

                    num_contributions += 1;
                }
            }

            ssao_blur_buffer.set(x, y, accum / num_contributions as f32);
        }
    }

    for y in 0..ssao_buffer.height {
        for x in 0..ssao_buffer.width {
            let stencil = stencil_buffer.0.get(x, y);

            if *stencil != 0 {
                ssao_buffer.set(x, y, *ssao_blur_buffer.get(x, y));
            }
        }
    }
}
