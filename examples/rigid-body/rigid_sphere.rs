use cairo::{
    matrix::Mat4, physics::simulation::rigid_body::RigidBody, transform::Transform3D,
    vec::vec3::Vec3,
};

fn get_moment_of_intertia_for_sphere(radius: f32, mass: f32) -> (Mat4, Mat4) {
    let scale = (2.0 / 5.0) * mass * radius * radius;

    let moment_of_inertia = Mat4::scale_uniform(scale);

    let inverse_moment_of_inertia = {
        let inverse_scale = 1.0 / scale;

        Mat4::scale_uniform(inverse_scale)
    };

    (moment_of_inertia, inverse_moment_of_inertia)
}

#[derive(Debug, Copy, Clone)]
pub struct RigidSphere {
    pub radius: f32,
    pub rigid_body: RigidBody,
}

impl Default for RigidSphere {
    fn default() -> Self {
        Self::new(Default::default(), 0.5, 1.0)
    }
}

impl RigidSphere {
    pub fn new(center: Vec3, radius: f32, mass: f32) -> Self {
        let (moment_of_inertia, inverse_moment_of_inertia) =
            get_moment_of_intertia_for_sphere(radius, mass);

        let mut transform = Transform3D::default();

        transform.set_translation(center);

        let rigid_body = RigidBody::new(
            mass,
            transform,
            moment_of_inertia,
            inverse_moment_of_inertia,
        );

        Self { radius, rigid_body }
    }
}
