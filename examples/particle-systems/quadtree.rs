use std::ptr::NonNull;

use arrayvec::ArrayVec;

use cairo::{physics::collision::aabb::AABB, vec::vec3::Vec3};

use crate::{force::Acceleration, simulation::universal_gravity_acceleration};

#[derive(Default, Debug, Clone)]
pub struct StaticParticle {
    mass: f32,
    position: Vec3,
}

const QUADTREE_LOAD_FACTOR: usize = 3;
const QUADTREE_LOAD_FACTOR_PLUS_ONE: usize = 4;

const TOP_LEFT_QUADRANT: usize = 0;
const TOP_RIGHT_QUADRANT: usize = 1;
const BOTTOM_LEFT_QUADRANT: usize = 2;
const BOTTOM_RIGHT_QUADRANT: usize = 3;

pub type QuadtreeLink = Option<NonNull<QuadtreeNode>>;

#[derive(Default, Debug, Clone)]
pub struct QuadtreeNode {
    pub bounds: AABB,
    pub center_of_mass: Vec3,
    pub bounding_radius: f32,
    pub total_mass: f32,
    pub particles: Option<ArrayVec<StaticParticle, QUADTREE_LOAD_FACTOR_PLUS_ONE>>,
    pub parent: QuadtreeLink,
    pub children: Option<[NonNull<QuadtreeNode>; 4]>,
}

impl QuadtreeNode {
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    pub fn is_internal(&self) -> bool {
        self.children.is_some()
    }

    pub fn contains_particles(&self) -> bool {
        self.is_internal() || self.particles.is_some()
    }

    pub fn acceleration(&self, point: &Vec3) -> Acceleration {
        static DISTANCE_THRESHOLD: f32 = 15.0;

        if self.is_leaf() {
            self.exact_acceleration(point)
        } else if !self.contains(point)
            && ((self.center_of_mass - *point).mag() - self.bounding_radius) > DISTANCE_THRESHOLD
        {
            self.approximate_acceleration(point)
        } else {
            let mut acceleration = Acceleration::default();

            for child_ptr in self.children.unwrap() {
                let child = unsafe { child_ptr.as_ref() };

                acceleration += child.acceleration(point);
            }

            acceleration
        }
    }

    fn exact_acceleration(&self, point: &Vec3) -> Acceleration {
        if let Some(particles) = &self.particles {
            let mut acceleration: Vec3 = Default::default();

            for particle in particles {
                if particle.position != *point {
                    acceleration +=
                        universal_gravity_acceleration(&particle.position, particle.mass, point);
                }
            }

            acceleration
        } else {
            Default::default()
        }
    }

    fn approximate_acceleration(&self, point: &Vec3) -> Acceleration {
        universal_gravity_acceleration(&self.center_of_mass, self.total_mass, point)
    }

    pub fn insert(&mut self, position: Vec3, mass: f32) {
        if self.is_leaf() {
            debug_assert!(self.children.is_none());

            match self.particles.as_mut() {
                Some(particles) => {
                    if particles.len() < QUADTREE_LOAD_FACTOR {
                        // Case 1. Current node is leaf node and not full yet.

                        // We can take this particle...

                        self.accumulate_particle(position, mass);
                    } else {
                        // Case 2. Current node is leaf node and will become
                        // full (need to split).

                        self.split(Some(StaticParticle { mass, position }));
                    }
                }
                None => {
                    self.accumulate_particle_new(position, mass);
                }
            }
        } else {
            // Node is internal. Walk the tree to find the smallest encompassing
            // leaf node.

            let mut current = self.get_node_for_position(position);

            current.insert(position, mass);

            while let Some(mut link) = current.parent {
                // Update accounting in internal ancestor node.

                let node = unsafe { link.as_mut() };

                node.accumulate_center_of_mass(position, mass);

                node.total_mass += mass;

                current = node;
            }
        }
    }

    fn split(&mut self, incoming_particle: Option<StaticParticle>) {
        let mut quadrants = self.quadrants();

        match self.particles.take() {
            Some(mut particles_to_move) => {
                // Migrate particles to their appropriate sub-quadrants.

                // Remember to include any newly inserted particle.

                match incoming_particle {
                    Some(particle) => particles_to_move.push(particle),
                    None => (),
                }

                for particle in particles_to_move {
                    // Determine which new (sub)quadrant this particle belongs in.

                    let quadrant_index = self.subquadrant_for(&particle.position);

                    // Push into the subquadrant.
                    quadrants[quadrant_index]
                        .accumulate_particle_new(particle.position, particle.mass);
                }
            }
            None => (),
        }

        self.children = Some([
            NonNull::new(Box::into_raw(Box::new(
                quadrants[TOP_LEFT_QUADRANT].to_owned(),
            )))
            .unwrap(),
            NonNull::new(Box::into_raw(Box::new(
                quadrants[TOP_RIGHT_QUADRANT].to_owned(),
            )))
            .unwrap(),
            NonNull::new(Box::into_raw(Box::new(
                quadrants[BOTTOM_LEFT_QUADRANT].to_owned(),
            )))
            .unwrap(),
            NonNull::new(Box::into_raw(Box::new(
                quadrants[BOTTOM_RIGHT_QUADRANT].to_owned(),
            )))
            .unwrap(),
        ]);
    }

    fn contains(&self, point: &Vec3) -> bool {
        point.x >= self.bounds.left
            && point.x <= self.bounds.right
            && point.y >= self.bounds.bottom
            && point.y <= self.bounds.top
    }

    fn subquadrant_for(&self, position: &Vec3) -> usize {
        // @NOTE: We won't update `self.bounds` until after this new particle
        // (position) is inserted into the appropriate (sub)quadrant.

        if position.x < self.bounds.center.x {
            // Particle belongs on the left half.
            if position.y < self.bounds.center.y {
                // Particle belongs in the bottom-left quadrant.
                BOTTOM_LEFT_QUADRANT
            } else {
                // Particle belongs in the top-left quadrant.
                TOP_LEFT_QUADRANT
            }
        } else {
            // Particle belongs on the right.
            if position.y < self.bounds.center.y {
                // Particle belongs in the bottom-right quadrant.
                BOTTOM_RIGHT_QUADRANT
            } else {
                // Particle belongs in the top-right quadrant.
                TOP_RIGHT_QUADRANT
            }
        }
    }

    fn recompute_bounding_radius(&mut self) {
        let center_of_mass_to_top_left = (Vec3 {
            x: self.bounds.left,
            y: self.bounds.top,
            z: 0.0,
        } - self.center_of_mass)
            .mag();

        let center_of_mass_to_top_right = (Vec3 {
            x: self.bounds.right,
            y: self.bounds.top,
            z: 0.0,
        } - self.center_of_mass)
            .mag();

        let center_of_mass_to_bottom_left = (Vec3 {
            x: self.bounds.left,
            y: self.bounds.bottom,
            z: 0.0,
        } - self.center_of_mass)
            .mag();

        let center_of_mass_to_bottom_right = (Vec3 {
            x: self.bounds.right,
            y: self.bounds.bottom,
            z: 0.0,
        } - self.center_of_mass)
            .mag();

        self.bounding_radius = center_of_mass_to_top_left
            .max(center_of_mass_to_top_right)
            .max(center_of_mass_to_bottom_left)
            .max(center_of_mass_to_bottom_right)
    }

    fn accumulate_particle_new(&mut self, position: Vec3, mass: f32) {
        self.particles = Some(ArrayVec::<StaticParticle, QUADTREE_LOAD_FACTOR_PLUS_ONE>::new());

        self.accumulate_particle(position, mass)
    }

    fn accumulate_particle(&mut self, position: Vec3, mass: f32) {
        if let Some(particles) = &mut self.particles {
            particles.push(StaticParticle { mass, position });

            debug_assert!(particles.len() <= QUADTREE_LOAD_FACTOR);

            if particles.len() == 1 {
                self.center_of_mass = position;

                self.recompute_bounding_radius();
            } else {
                self.accumulate_center_of_mass(position, mass);
            }

            self.total_mass += mass;
        } else {
            panic!("Called QuadtreeNode::accumulate_particle() on node with no particles list!")
        }
    }

    fn accumulate_center_of_mass(&mut self, position: Vec3, mass: f32) {
        // @NOTE: Assumes that `self.total_mass` is not yet updated to reflect
        // the mass contributed by this new particle.

        let sum_of_weighted_position = self.center_of_mass * self.total_mass;

        self.center_of_mass =
            (sum_of_weighted_position + position * mass) / (self.total_mass + mass);

        self.recompute_bounding_radius()
    }

    fn quadrants(&mut self) -> Vec<QuadtreeNode> {
        self.bounds
            .subdivide_2d()
            .iter()
            .map(|sub| {
                let node = QuadtreeNode {
                    bounds: *sub,
                    center_of_mass: sub.center,
                    parent: Some(unsafe { NonNull::new_unchecked(self) }),
                    ..Default::default()
                };

                node
            })
            .collect()
    }

    fn get_node_for_position(&mut self, position: Vec3) -> &mut Self {
        let mut current = self;

        while current.is_internal() {
            let quadrant_index = current.subquadrant_for(&position);

            match current.children.as_mut() {
                Some(children) => {
                    current = unsafe { children[quadrant_index].as_mut() };
                }
                None => panic!("Something is very wrong!"),
            }
        }

        current
    }
}

#[derive(Default, Debug)]
pub struct Quadtree {
    pub root: QuadtreeLink,
}

impl Quadtree {
    pub fn new(extent: (Vec3, Vec3)) -> Self {
        let bounds = AABB::from_min_max(extent.0, extent.1);

        let mut tree = Self::default();

        let root_node = QuadtreeNode {
            bounds,
            ..Default::default()
        };

        let root_node_boxed = Box::new(root_node);
        let root_node_ptr = Box::into_raw(root_node_boxed);
        let non_null = NonNull::new(root_node_ptr);

        tree.root = non_null;

        tree
    }
}
