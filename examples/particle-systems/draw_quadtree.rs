use cairo::{
    buffer::Buffer2D,
    color::{self, Color},
    graphics::Graphics,
    vec::vec3::Vec3,
};

use crate::{
    coordinates::world_to_screen_space,
    quadtree::{Quadtree, QuadtreeNode},
};

// Theme:
// https://color.adobe.com/Teals-color-theme-19171364

static COLORS: [Color; 5] = [
    Color {
        r: 17.0,
        g: 186.0,
        b: 184.0,
        a: 1.0,
    },
    Color {
        r: 15.0,
        g: 161.0,
        b: 158.0,
        a: 1.0,
    },
    Color {
        r: 0.0,
        g: 132.0,
        b: 129.0,
        a: 1.0,
    },
    Color {
        r: 4.0,
        g: 79.0,
        b: 78.0,
        a: 1.0,
    },
    Color {
        r: 6.0,
        g: 47.0,
        b: 47.0,
        a: 1.0,
    },
];

fn draw_quadtree_node(
    node: &QuadtreeNode,
    buffer: &mut Buffer2D,
    framebuffer_center: &Vec3,
    depth: usize,
    parent_center_of_mass_screen_space: Option<(f32, f32)>,
) {
    let node_bounds = node.bounds;

    let (x, y, width, height) = {
        let top_left_world_space = Vec3 {
            x: node_bounds.left,
            y: node_bounds.top,
            z: 0.0,
        };

        let bottom_right_world_space = Vec3 {
            x: node_bounds.right,
            y: node_bounds.bottom,
            z: 0.0,
        };

        let top_left_screen_space =
            world_to_screen_space(&top_left_world_space, framebuffer_center);

        let bottom_right_screen_space =
            world_to_screen_space(&bottom_right_world_space, framebuffer_center);

        let x = top_left_screen_space.x;
        let y = top_left_screen_space.y;
        let width = bottom_right_screen_space.x - x;
        let height = bottom_right_screen_space.y - y;

        (x, y, width, height)
    };

    let center_of_mass_screen_space =
        world_to_screen_space(&node.center_of_mass, framebuffer_center);

    let (center_of_mass_x, center_of_mass_y) = (
        center_of_mass_screen_space.x as i32,
        center_of_mass_screen_space.y as i32,
    );

    if let Some((parent_center_of_mass_x, parent_center_of_mass_y)) =
        parent_center_of_mass_screen_space
    {
        // Draw a (clipped) line from this node's center-of-mass to the parent's
        // center-of-mass.

        Graphics::line(
            buffer,
            center_of_mass_x,
            center_of_mass_y,
            parent_center_of_mass_x as i32,
            parent_center_of_mass_y as i32,
            if node.contains_particles() {
                &color::YELLOW
            } else {
                &color::DARK_GRAY
            },
        );
    }

    // Check whether this node overlaps the buffer's bounds.

    if let Some((safe_x, safe_y, safe_width, safe_height)) =
        Graphics::clip_rectangle(x as i32, y as i32, width as u32, height as u32, buffer)
    {
        let color_for_depth = COLORS[depth % 4] * 0.33;
        let color_for_depth_u32 = color_for_depth.to_u32();

        // Draw quadrant borders.

        // Left vertical border.
        if safe_x as i32 == x as i32 {
            for y in safe_y..safe_y + safe_height + 1 {
                buffer.set(safe_x, y, color_for_depth_u32);
            }
        }

        // Right vertical border.
        if (safe_x + safe_width) as i32 == (x + width) as i32 {
            for y in safe_y..safe_y + safe_height + 1 {
                let x = safe_x + safe_width;
                buffer.set(x, y, color_for_depth_u32);
            }
        }

        // Top horizontal border.
        if safe_y as i32 == y as i32 {
            for x in safe_x..safe_x + safe_width + 1 {
                buffer.set(x, safe_y, color_for_depth_u32);
            }
        }

        // Bottom horizontal border.
        if (safe_y + safe_height) as i32 == (y + height) as i32 {
            for x in safe_x..safe_x + safe_width + 1 {
                let y = safe_y + safe_height;
                buffer.set(x, y, color_for_depth_u32);
            }
        }

        static CENTER_OF_MASS_INDICATOR_SIZE: u32 = 8;
        static CENTER_OF_MASS_INDICATOR_SIZE_OVER_2: u32 = CENTER_OF_MASS_INDICATOR_SIZE - 2;

        if let Some((x, y, width, height)) = Graphics::clip_rectangle(
            center_of_mass_x - CENTER_OF_MASS_INDICATOR_SIZE_OVER_2 as i32,
            center_of_mass_y - CENTER_OF_MASS_INDICATOR_SIZE_OVER_2 as i32,
            CENTER_OF_MASS_INDICATOR_SIZE,
            CENTER_OF_MASS_INDICATOR_SIZE,
            buffer,
        ) {
            // Draw quadrant's center of mass.

            Graphics::rectangle(
                buffer,
                x,
                y,
                width,
                height,
                None,
                Some(if node.contains_particles() {
                    &color::YELLOW
                } else {
                    &color::DARK_GRAY
                }),
            )
        }

        // Draw any in-bounds subquadrants.

        if let Some(children) = node.children {
            for link in children.iter() {
                let child = unsafe { link.as_ref() };

                draw_quadtree_node(
                    child,
                    buffer,
                    framebuffer_center,
                    depth + 1,
                    Some((center_of_mass_screen_space.x, center_of_mass_screen_space.y)),
                );
            }
        }
    }
}

pub fn draw_quadtree(tree: &Quadtree, buffer: &mut Buffer2D, framebuffer_center: &Vec3) {
    match tree.root {
        Some(ptr) => unsafe {
            let root = ptr.as_ref();

            draw_quadtree_node(root, buffer, framebuffer_center, 0, None);
        },
        None => {
            // Tree is empty (no particles exist in the current state).
        }
    }
}
