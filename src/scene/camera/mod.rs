use std::{f32::consts::PI, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    serde::PostDeserialize,
    shader::context::ShaderContext,
    time::TimingInfo,
    transform::look_vector::LookVector,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use self::frustum::{Frustum, FAR_PLANE_POINTS_CLIP_SPACE, NEAR_PLANE_POINTS_CLIP_SPACE};

pub mod frustum;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraProjectionKind {
    #[default]
    Perspective,
    Orthographic,
}

impl fmt::Display for CameraProjectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CameraProjectionKind::Perspective => "Perspective",
                CameraProjectionKind::Orthographic => "Orthographic",
            }
        )
    }
}

pub static CAMERA_PROJECTION_KINDS: [CameraProjectionKind; 2] = [
    CameraProjectionKind::Perspective,
    CameraProjectionKind::Orthographic,
];

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CameraOrthographicExtent {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

static DEFAULT_CAMERA_FIELD_OF_VIEW: f32 = 75.0;
static DEFAULT_CAMERA_ASPECT_RATIO: f32 = 16.0 / 9.0;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub is_active: bool,
    kind: CameraProjectionKind,
    field_of_view: Option<f32>,
    aspect_ratio: Option<f32>,
    extent: Option<CameraOrthographicExtent>,
    pub movement_speed: f32,
    projection_z_near: f32,
    projection_z_far: f32,
    #[serde(skip)]
    projection_transform: Mat4,
    #[serde(skip)]
    projection_inverse_transform: Mat4,
    pub look_vector: LookVector,
    #[serde(skip)]
    frustum: Frustum,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            CameraProjectionKind::Perspective,
            Default::default(),
            vec3::FORWARD,
            Some(90.0),
            Some(16.0 / 9.0),
            None,
        )
    }
}

impl PostDeserialize for Camera {
    fn post_deserialize(&mut self) {
        self.recompute_projections();
        self.recompute_world_space_frustum();
    }
}

impl fmt::Display for Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Camera (kind={}, fov={}, aspect_ratio={}, z_near={}, z_far={})",
            self.kind,
            match self.field_of_view {
                Some(fov) => fov.to_string(),
                None => "n/a".to_string(),
            },
            match self.aspect_ratio {
                Some(ratio) => ratio.to_string(),
                None => "n/a".to_string(),
            },
            self.projection_z_near,
            self.projection_z_far
        )
    }
}

impl Camera {
    pub fn new(
        kind: CameraProjectionKind,
        position: Vec3,
        target: Vec3,
        field_of_view: Option<f32>,
        aspect_ratio: Option<f32>,
        extent: Option<CameraOrthographicExtent>,
    ) -> Self {
        let projection_z_near = 0.3;
        let projection_z_far = 1000.0;

        let mut camera = Camera {
            is_active: false,
            kind,
            field_of_view,
            aspect_ratio,
            extent,
            projection_z_near,
            projection_z_far,
            movement_speed: 50.0,
            projection_transform: Default::default(),
            projection_inverse_transform: Default::default(),
            look_vector: LookVector::new(position, target),
            frustum: Default::default(),
        };

        camera.post_deserialize();

        camera
    }

    pub fn from_orthographic(
        position: Vec3,
        target: Vec3,
        extent: CameraOrthographicExtent,
    ) -> Self {
        Camera::new(
            CameraProjectionKind::Orthographic,
            position,
            target,
            None,
            None,
            Some(extent),
        )
    }

    pub fn from_perspective(
        position: Vec3,
        target: Vec3,
        field_of_view: f32,
        aspect_ratio: f32,
    ) -> Self {
        Camera::new(
            CameraProjectionKind::Perspective,
            position,
            target,
            Some(field_of_view),
            Some(aspect_ratio),
            None,
        )
    }

    pub fn get_kind(&self) -> CameraProjectionKind {
        self.kind
    }

    pub fn set_kind(&mut self, kind: CameraProjectionKind) {
        self.kind = kind;

        self.recompute_projections();

        self.recompute_world_space_frustum();
    }

    pub fn get_field_of_view(&self) -> Option<f32> {
        self.field_of_view
    }

    pub fn set_field_of_view(&mut self, fov: Option<f32>) {
        self.field_of_view = fov;

        self.recompute_projections();

        self.recompute_world_space_frustum();
    }

    pub fn get_aspect_ratio(&self) -> Option<f32> {
        self.aspect_ratio
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) -> Result<(), String> {
        match self.kind {
            CameraProjectionKind::Perspective => {
                self.aspect_ratio = Some(aspect_ratio);

                self.recompute_projections();

                self.recompute_world_space_frustum();

                Ok(())
            }
            CameraProjectionKind::Orthographic => {
                Err("Called Camera::set_aspect_ratio() on an orthographic camera!".to_string())
            }
        }
    }

    pub fn get_projection_z_near(&self) -> f32 {
        self.projection_z_near
    }

    pub fn set_projection_z_near(&mut self, near: f32) {
        self.projection_z_near = near;

        self.recompute_projections();

        self.recompute_world_space_frustum();
    }

    pub fn get_projection_z_far(&self) -> f32 {
        self.projection_z_far
    }

    pub fn set_projection_z_far(&mut self, far: f32) {
        self.projection_z_far = far;

        self.recompute_projections();

        self.recompute_world_space_frustum();
    }

    pub fn get_frustum(&self) -> &Frustum {
        &self.frustum
    }

    pub fn get_view_transform(&self) -> Mat4 {
        Mat4::look_at_inverse(
            self.look_vector.get_position(),
            self.look_vector.get_forward(),
            self.look_vector.get_right(),
            self.look_vector.get_up(),
        )
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
        Mat4::tbn(
            self.look_vector.get_right(),
            self.look_vector.get_up(),
            self.look_vector.get_forward(),
        )
    }

    pub fn get_projection(&self) -> Mat4 {
        self.projection_transform
    }

    pub fn get_projection_inverse(&self) -> Mat4 {
        self.projection_inverse_transform
    }

    pub fn recompute_world_space_frustum(&mut self) {
        // Canonical (clip space) view volume.

        let (near_plane_points_world_space, far_plane_points_world_space) = match self.get_kind() {
            CameraProjectionKind::Perspective => {
                let fov = self.get_field_of_view().unwrap();
                let fov_rad = fov * PI / 180.0;

                let opposite_over_adjacent_x = (fov_rad / 2.0).tan();

                let opposite_over_adjacent_y =
                    opposite_over_adjacent_x / self.get_aspect_ratio().unwrap();

                let (near, far) = (self.get_projection_z_near(), self.get_projection_z_far());

                let near_plane_points_world_space = NEAR_PLANE_POINTS_CLIP_SPACE
                    .map(|mut coord| {
                        coord.x *= near * opposite_over_adjacent_x;
                        coord.y *= near * opposite_over_adjacent_y;
                        coord.z = near;

                        coord * self.get_view_transform()
                    })
                    .map(|coord| coord.to_vec3());

                let far_plane_points_world_space = FAR_PLANE_POINTS_CLIP_SPACE
                    .map(|mut coord| {
                        coord.x *= far * opposite_over_adjacent_x;
                        coord.y *= far * opposite_over_adjacent_y;
                        coord.z = far;

                        coord * self.get_view_transform()
                    })
                    .map(|coord| coord.to_vec3());

                (near_plane_points_world_space, far_plane_points_world_space)
            }
            CameraProjectionKind::Orthographic => {
                let near_plane_points_world_space = NEAR_PLANE_POINTS_CLIP_SPACE
                    .map(|coord| coord * self.get_projection_inverse() * self.get_view_transform())
                    .map(|coord| coord.to_vec3());

                let far_plane_points_world_space = FAR_PLANE_POINTS_CLIP_SPACE
                    .map(|coord| coord * self.get_projection_inverse() * self.get_view_transform())
                    .map(|coord| coord.to_vec3());

                (near_plane_points_world_space, far_plane_points_world_space)
            }
        };

        self.frustum = Frustum {
            forward: self.look_vector.get_forward(),
            near: near_plane_points_world_space,
            far: far_plane_points_world_space,
        };
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

        // Pixel coordinate in world-space.

        pixel_coordinate_camera_view_space * self.get_view_rotation_transform()
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
            match self.kind {
                CameraProjectionKind::Perspective => Some(mouse_state),
                CameraProjectionKind::Orthographic => None,
            },
            game_controller_state,
            self.movement_speed,
        );

        // Apply field-of-view zoom based on mousewheel input.

        if let Some(event) = &mouse_state.wheel_event {
            match self.kind {
                CameraProjectionKind::Perspective => {
                    self.set_projection_z_far(self.get_projection_z_far() + event.delta as f32);

                    self.recompute_projections();

                    self.recompute_world_space_frustum();
                }
                CameraProjectionKind::Orthographic => {
                    let current_z_far = self.get_projection_z_far();

                    self.set_projection_z_far(current_z_far + event.delta as f32);
                }
            }
        }

        self.recompute_world_space_frustum();
    }

    pub fn update_shader_context(&self, ctx: &mut ShaderContext) {
        ctx.set_view_position(Vec4::new(self.look_vector.get_position(), 1.0));

        ctx.set_view_inverse_transform(self.get_view_inverse_transform());

        ctx.set_projection(self.get_projection());
    }

    fn recompute_projections(&mut self) {
        match self.kind {
            CameraProjectionKind::Perspective => {
                let field_of_view = match self.field_of_view {
                    Some(fov) => fov,
                    None => DEFAULT_CAMERA_FIELD_OF_VIEW,
                };

                let aspect_ratio = match self.aspect_ratio {
                    Some(aspect_ratio) => aspect_ratio,
                    None => DEFAULT_CAMERA_ASPECT_RATIO,
                };

                self.projection_transform = Mat4::perspective_for_fov(
                    field_of_view,
                    aspect_ratio,
                    self.projection_z_near,
                    self.projection_z_far,
                );

                self.projection_inverse_transform = Mat4::perspective_inverse_for_fov(
                    field_of_view,
                    aspect_ratio,
                    self.projection_z_near,
                    self.projection_z_far,
                );
            }
            CameraProjectionKind::Orthographic => {
                let extent = self.extent.unwrap();

                let (left, right, bottom, top, near, far) = (
                    extent.left,
                    extent.right,
                    extent.bottom,
                    extent.top,
                    self.projection_z_near,
                    self.projection_z_far,
                );

                self.projection_transform = Mat4::orthographic(left, right, bottom, top, near, far);

                self.projection_inverse_transform =
                    Mat4::orthographic_inverse(left, right, bottom, top, near, far);
            }
        }
    }
}
