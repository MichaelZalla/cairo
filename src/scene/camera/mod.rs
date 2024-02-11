use crate::{
    device::{GameControllerState, KeyboardState, MouseState},
    matrix::Mat4,
    time::TimingInfo,
    transform::look_vector::LookVector,
    vec::{vec3::Vec3, vec4::Vec4},
};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub movement_speed: f32,
    field_of_view: f32,
    aspect_ratio: f32,
    projection_z_near: f32,
    projection_z_far: f32,
    projection_transform: Mat4,
    projection_inverse_transform: Mat4,
    pub look_vector: LookVector,
}

impl Camera {
    pub fn new(aspect_ratio: f32, position: Vec3, target: Vec3) -> Self {
        let field_of_view = 75.0;

        let projection_z_near = 0.3;
        let projection_z_far = 1000.0;

        let projection_transform = Mat4::perspective_for_fov(
            field_of_view,
            aspect_ratio,
            projection_z_near,
            projection_z_far,
        );

        let projection_inverse_transform = Mat4::perspective_inverse_for_fov(
            field_of_view,
            aspect_ratio,
            projection_z_near,
            projection_z_far,
        );

        let camera = Camera {
            field_of_view,
            aspect_ratio,
            projection_z_near,
            projection_z_far,
            movement_speed: 50.0,
            projection_transform,
            projection_inverse_transform,
            look_vector: LookVector::new(position, target),
        };

        return camera;
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn get_projection_z_near(&self) -> f32 {
        self.projection_z_near
    }

    pub fn get_projection_z_far(&self) -> f32 {
        self.projection_z_far
    }

    pub fn set_projection_z_far(&mut self, far: f32) {
        self.projection_z_far = far;

        self.projection_transform = Mat4::perspective_for_fov(
            self.field_of_view,
            self.aspect_ratio,
            self.projection_z_near,
            self.projection_z_far,
        );

        self.projection_inverse_transform = Mat4::perspective_inverse_for_fov(
            self.field_of_view,
            self.aspect_ratio,
            self.projection_z_near,
            self.projection_z_far,
        );
    }

    pub fn get_view_inverse_transform(&self) -> Mat4 {
        Mat4::look_at(
            self.look_vector.get_position(),
            self.look_vector.get_forward(),
            self.look_vector.get_right(),
            self.look_vector.get_up(),
        )
    }

    pub fn get_view_rotation_transform(&self) -> Mat4 {
        let (f, r, u) = (
            self.look_vector.get_forward(),
            self.look_vector.get_right(),
            self.look_vector.get_up(),
        );

        Mat4::new_from_elements([
            // Row-major ordering
            [r.x, r.y, r.z, 0.0],
            [u.x, u.y, u.z, 0.0],
            [f.x, f.y, f.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn get_projection(&self) -> Mat4 {
        self.projection_transform
    }

    pub fn get_projection_inverse(&self) -> Mat4 {
        self.projection_inverse_transform
    }

    pub fn get_near_plane_pixel_world_space_position(
        &self,
        screen_x: u32,
        screen_y: u32,
        width: u32,
        height: u32,
    ) -> Vec4 {
        let pixel_coordinate_ndc_space = Vec4 {
            x: screen_x as f32 / width as f32,
            y: screen_y as f32 / height as f32,
            z: -1.0,
            w: 1.0,
        };

        // Transform our screen-space coordinate by the camera's inverse projection.

        let pixel_coordinate_projection_space =
            pixel_coordinate_ndc_space * self.get_projection_inverse();

        // Camera-direction vector in camera-view-space: (0, 0, -1)

        // Compute pixel coordinate in camera-view-space.

        // Near-plane coordinates in camera-view-space:
        //
        //  x: -1 to 1
        //  y: -1 to 1 (y is up)
        //  z: -1 (near) to 1 (far)

        let pixel_coordinate_camera_view_space: Vec4 = Vec4 {
            x: -1.0 + pixel_coordinate_projection_space.x * 2.0,
            y: -1.0 + (1.0 - pixel_coordinate_projection_space.y) * 2.0,
            z: 1.0,
            w: 1.0, // ???????
        };

        // Transform camera-view-space coordinate to world-space coordinate.

        // Note: Treating the camera's position as the world-space origin.

        let pixel_coordinate_world_space =
            pixel_coordinate_camera_view_space * self.get_view_rotation_transform();

        pixel_coordinate_world_space
    }

    pub fn update(
        &mut self,
        timing_info: &TimingInfo,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        self.look_vector.update(
            timing_info,
            keyboard_state,
            mouse_state,
            game_controller_state,
            self.movement_speed,
        );

        // Apply field-of-view zoom based on mousewheel input.

        if mouse_state.wheel_did_move {
            self.field_of_view -= mouse_state.wheel_y as f32;

            self.field_of_view = self.field_of_view.max(1.0).min(120.0);

            self.projection_transform = Mat4::perspective_for_fov(
                self.field_of_view,
                self.aspect_ratio,
                self.projection_z_near,
                self.projection_z_far,
            );

            self.projection_inverse_transform = Mat4::perspective_inverse_for_fov(
                self.field_of_view,
                self.aspect_ratio,
                self.projection_z_near,
                self.projection_z_far,
            );
        }
    }
}
