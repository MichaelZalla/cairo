use std::fmt;

use crate::{
    geometry::primitives::plane::Plane,
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision::RESTING_SPEED_THRESHOLD,
            rigid_body::rigid_body_simulation_state::RigidBodySimulationState,
        },
    },
    vec::vec3::Vec3,
};

// Distance threshold for detecting static contact.
// Bodies within this distance of a surface may be considered in contact.
// Increased to 0.2 to ensure contacts are detected after collision response
pub const CONTACT_DISTANCE_THRESHOLD: f32 = 0.2;

// Ratio threshold for rolling detection. Contact velocity must be less than
// this ratio of body velocity to be considered rolling.
const ROLLING_VELOCITY_RATIO_THRESHOLD: f32 = 0.1;

// Minimum body velocity for rolling detection.
const BODY_VELOCITY_ROLLING_THRESHOLD: f32 = 0.01;

// Minimum contact point velocity for rolling detection.
const CONTACT_POINT_VELOCITY_ROLLING_THRESHOLD: f32 = 0.2;

#[derive(Default, Debug, Copy, Clone)]
pub enum StaticContactKind {
    #[default]
    Resting,
    Sliding,
    Rolling(f32),
}

impl fmt::Display for StaticContactKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Resting => "Resting".to_string(),
                Self::Sliding => "Sliding".to_string(),
                Self::Rolling(rolling_resistance) => format!("Rolling({})", rolling_resistance),
            }
        )
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct StaticContact {
    pub kind: StaticContactKind,
    pub point: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub contact_point_velocity: Vec3,
    pub material: PhysicsMaterial,
    pub penetration_depth: f32, // Positive = penetrating, negative = gap/separation
}

#[derive(Debug, Copy, Clone)]
pub struct StaticContactList<const N: usize> {
    pub static_contacts: [StaticContact; N],
    pub len: usize,
}

impl<const N: usize> Default for StaticContactList<N> {
    fn default() -> Self {
        Self {
            static_contacts: [Default::default(); N],
            len: 0,
        }
    }
}

impl<const N: usize> StaticContactList<N> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn iter(&self) -> Iter<'_, N> {
        Iter {
            len: self.len,
            current_index: 0,
            list: self,
        }
    }

    pub fn push(&mut self, contact: StaticContact) -> Result<(), String> {
        if self.len == N {
            return Err("Overflow.".to_string());
        }

        self.static_contacts[self.len] = contact;

        self.len += 1;

        Ok(())
    }
}

pub struct Iter<'a, const N: usize> {
    list: &'a StaticContactList<N>,
    current_index: usize,
    len: usize,
}

impl<'a, const N: usize> Iterator for Iter<'a, N> {
    type Item = &'a StaticContact;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.list.len {
            return None;
        }

        let contact = &self.list.static_contacts[self.current_index];

        self.current_index += 1;

        Some(contact)
    }
}

impl<const N: usize> ExactSizeIterator for Iter<'_, N> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, const N: usize> IntoIterator for &'a StaticContactList<N> {
    type Item = &'a StaticContact;

    type IntoIter = Iter<'a, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IntoIter<const N: usize> {
    list: StaticContactList<N>,
    current_index: usize,
}

impl<const N: usize> Iterator for IntoIter<N> {
    type Item = StaticContact;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.list.len {
            return None;
        }

        let contact = self.list.static_contacts[self.current_index];

        self.current_index += 1;

        Some(contact)
    }
}

impl<const N: usize> ExactSizeIterator for IntoIter<N> {
    fn len(&self) -> usize {
        self.list.len - self.current_index
    }
}

impl<const N: usize> IntoIterator for StaticContactList<N> {
    type Item = StaticContact;

    type IntoIter = IntoIter<N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            list: self,
            current_index: 0,
        }
    }
}

/// Computes the linear velocity of a contact point on a rigid body.
///
/// This accounts for both the body's linear velocity and the velocity
/// induced by rotation at the contact point.
pub fn compute_contact_point_velocity(
    body_velocity: Vec3,
    angular_velocity: Vec3,
    contact_point: Vec3,
    body_position: Vec3,
) -> Vec3 {
    let r = contact_point - body_position;
    body_velocity + angular_velocity.cross(r)
}

/// Computes the motion tangent for a velocity projected onto a plane.
///
/// Returns the normalized direction of motion along the plane surface,
/// or None if there is no tangential motion.
pub fn compute_motion_tangent(point_on_plane: Vec3, velocity: Vec3, plane: &Plane) -> Option<Vec3> {
    let v = point_on_plane + velocity;
    let d = plane.get_signed_distance(&v);
    let velocity_projected_onto_plane = v - plane.normal * d;

    if velocity_projected_onto_plane.is_zero() {
        None
    } else {
        Some(velocity_projected_onto_plane.as_normal())
    }
}

/// Classifies the kind of static contact based on velocities and spin.
///
/// Distinguishes between:
/// - Resting: Both body and contact point have minimal velocity
/// - Rolling: Contact point velocity is small relative to body velocity, with correct spin axis
/// - Sliding: Contact point has significant tangential velocity
pub fn classify_contact_kind(
    body_velocity: Vec3,
    contact_point_velocity: Vec3,
    body_motion_tangent: Option<Vec3>,
    contact_point_motion_tangent: Option<Vec3>,
    angular_velocity: Vec3,
    normal: Vec3,
) -> StaticContactKind {
    // Constants for motion/spin significance thresholds
    const MIN_VELOCITY_FOR_AXIS_CHECK: f32 = 0.01;

    const MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK: f32 = 0.01;

    const MIN_VELOCITY_FOR_AXIS_CHECK_SQUARED: f32 =
        MIN_VELOCITY_FOR_AXIS_CHECK * MIN_VELOCITY_FOR_AXIS_CHECK;

    const MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK_SQUARED: f32 =
        MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK * MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK;

    let body_speed_along_tangent = if let Some(tangent) = body_motion_tangent {
        body_velocity.dot(tangent)
    } else {
        0.0
    };

    let contact_point_speed_along_tangent = if let Some(tangent) = contact_point_motion_tangent {
        contact_point_velocity.dot(tangent)
    } else {
        0.0
    };

    // Early return for resting: if both velocities are very small, body is at rest.
    if body_speed_along_tangent < RESTING_SPEED_THRESHOLD
        && contact_point_speed_along_tangent < RESTING_SPEED_THRESHOLD
    {
        return StaticContactKind::Resting;
    }

    // Check if contact velocity is small relative to body velocity (rolling)
    let velocity_ratio = if body_speed_along_tangent > BODY_VELOCITY_ROLLING_THRESHOLD {
        contact_point_speed_along_tangent / body_speed_along_tangent
    } else {
        // At very low speeds, avoid division by near-zero
        if contact_point_speed_along_tangent < CONTACT_POINT_VELOCITY_ROLLING_THRESHOLD {
            0.0 // Treat as rolling
        } else {
            1.0 // Treat as sliding
        }
    };

    let is_rolling = velocity_ratio < ROLLING_VELOCITY_RATIO_THRESHOLD;

    // Additional check for rolling: the spin axis must be orthogonal to
    // both (1) the surface normal, and (2) the direction of motion.
    let has_correct_spin_axis = {
        let has_significant_motion =
            body_velocity.mag_squared() > MIN_VELOCITY_FOR_AXIS_CHECK_SQUARED;

        let has_significant_spin =
            angular_velocity.mag_squared() > MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK_SQUARED;

        if has_significant_motion && has_significant_spin {
            let spin_axis = angular_velocity.as_normal();
            let motion_direction = body_velocity.as_normal();

            // Spin must be perpendicular to normal.
            let dot_with_normal = normal.dot(spin_axis).abs();
            let perpendicular_to_normal = dot_with_normal < 0.1;

            // Spin must be perpendicular to motion.
            let dot_with_motion = motion_direction.dot(spin_axis);
            let perpendicular_to_motion = dot_with_motion.abs() < 0.1;

            perpendicular_to_motion && perpendicular_to_normal
        } else if has_significant_spin {
            // High spin but low tangential velocity: check if spin is parallel
            // to surface normal (i.e., Y-spin); if so, this is definitely NOT rolling.
            let spin_axis = angular_velocity.as_normal();
            let dot_with_normal = normal.dot(spin_axis).abs();
            let parallel_to_normal = dot_with_normal > 0.9;

            !parallel_to_normal
        } else {
            // Low velocity AND low spin - can't determine axis, allow resting/rolling
            true
        }
    };

    if is_rolling && has_correct_spin_axis {
        // println!("[CONTACT CLASSIFICATION] Rolling detected: body_speed={:.3}, contact_speed={:.3}, ratio={:.3}",
        //          body_speed_along_tangent, contact_point_speed_along_tangent, velocity_ratio);
        StaticContactKind::Rolling(0.5)
    } else {
        // if !is_rolling {
        //     println!("[CONTACT CLASSIFICATION] Not rolling - velocity_ratio check failed: ratio={:.3}, threshold={:.3}",
        //              velocity_ratio, ROLLING_VELOCITY_RATIO_THRESHOLD);
        // } else {
        //     println!("[CONTACT CLASSIFICATION] Not rolling - spin axis check failed");
        // }
        StaticContactKind::Sliding
    }
}

/// Computes a contact basis (tangent and bitangent) for the contact normal.
///
/// If a motion tangent is provided, it's used as the primary tangent direction.
/// Otherwise, an arbitrary perpendicular vector is chosen.
pub fn compute_contact_basis(
    normal: Vec3,
    contact_point_motion_tangent: Option<Vec3>,
) -> (Vec3, Vec3) {
    // // Validate that normal is normalized
    // assert!(
    //     (normal.mag() - 1.0).abs() < 1e-4,
    //     "Normal must be normalized (mag: {})",
    //     normal.mag()
    // );

    // // Validate that motion tangent is normalized (if provided)
    // if let Some(t) = contact_point_motion_tangent {
    //     assert!(
    //         (t.mag() - 1.0).abs() < 1e-4,
    //         "Motion tangent must be normalized (mag: {})",
    //         t.mag()
    //     );
    // }

    let tangent = match contact_point_motion_tangent {
        Some(t) => {
            // Orthogonalize: remove component parallel to normal using Gram-Schmidt
            let t_perpendicular = t - normal * t.dot(normal);

            // Check if motion tangent is (nearly) parallel to normal
            if t_perpendicular.mag() < 1e-6 {
                // Motion tangent is parallel to normal, use arbitrary perpendicular
                normal.basis().1
            } else {
                // Normalize the orthogonalized tangent
                t_perpendicular.as_normal()
            }
        }
        None => {
            // No motion tangent, use any perpendicular vector
            normal.basis().1
        }
    };

    // Compute bitangent and normalize it
    let bitangent = normal.cross(tangent).as_normal();

    (tangent, bitangent)
}

/// Detects and creates a static contact if conditions are met.
///
/// This function performs the full contact detection logic:
/// 1. Checks if the body is close enough to the surface
/// 2. Verifies the contact point velocity along the normal is small
/// 3. Optionally checks if forces are pulling the body away
/// 4. Classifies the contact kind (resting, rolling, or sliding)
#[allow(clippy::too_many_arguments)]
pub fn get_static_contact(
    derivative: &RigidBodySimulationState,
    new_body_state: &RigidBodySimulationState,
    surface_normal: Vec3,
    signed_distance: f32,
    minimum_distance: f32,
    body_motion_tangent: Option<Vec3>,
    contact_point_motion_tangent: Option<Vec3>,
    material: &PhysicsMaterial,
    check_forces: bool,
) -> Option<StaticContact> {
    // 1. Requires that the body be very close to the collider
    if signed_distance > minimum_distance + CONTACT_DISTANCE_THRESHOLD {
        // println!("[CONTACT REJECTION] Distance check failed: signed_dist={:.3}, min_dist={:.3}, threshold={:.3}",
        //          signed_distance, minimum_distance, CONTACT_DISTANCE_THRESHOLD);
        return None;
    }

    // 2. Requires that the contact point's speed along the normal be very small
    let body_velocity = new_body_state.velocity();
    let contact_point = new_body_state.position - surface_normal * minimum_distance;

    let contact_point_velocity = compute_contact_point_velocity(
        body_velocity,
        new_body_state.angular_velocity(),
        contact_point,
        new_body_state.position,
    );

    let contact_point_speed_along_normal = contact_point_velocity.dot(surface_normal);

    if contact_point_speed_along_normal.abs() > RESTING_SPEED_THRESHOLD {
        return None;
    }

    // 3. (Optional) Requires that the forces acting on the body aren't
    //    pulling the body away from the collider.
    if check_forces {
        let magnitude_of_acceleration_along_normal = derivative.linear_momentum.dot(surface_normal);

        if magnitude_of_acceleration_along_normal > 0.01 {
            // println!("[CONTACT REJECTION] Force check failed: accel_normal={:.3}, threshold=0.01",
            //          magnitude_of_acceleration_along_normal);
            return None;
        }
    }

    // At this point, we can be certain that the object is resting, rolling, or sliding

    let angular_velocity = new_body_state.angular_velocity();

    let kind = classify_contact_kind(
        body_velocity,
        contact_point_velocity,
        body_motion_tangent,
        contact_point_motion_tangent,
        angular_velocity,
        surface_normal,
    );

    let (tangent, bitangent) = compute_contact_basis(surface_normal, contact_point_motion_tangent);

    // Calculate penetration depth: positive = penetrating, negative = gap
    let penetration_depth = minimum_distance - signed_distance;

    Some(StaticContact {
        kind,
        point: contact_point,
        normal: surface_normal,
        tangent,
        bitangent,
        contact_point_velocity,
        material: *material,
        penetration_depth,
    })
}
