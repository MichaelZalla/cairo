use std::{collections::HashMap, ops};

use crate::{
    color::Color,
    geometry::primitives::aabb::Bounded,
    matrix::Mat4,
    render::Renderer,
    scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer,
    vec::vec3::{self, Vec3},
};

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct GridSpaceCoordinate {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl ops::Add for GridSpaceCoordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl From<(Vec3, f32)> for GridSpaceCoordinate {
    fn from(vec_and_scale: (Vec3, f32)) -> Self {
        let (vec, scale) = vec_and_scale;

        Self {
            x: (vec.x / scale).floor() as isize,
            y: (vec.y / scale).floor() as isize,
            z: (vec.z / scale).floor() as isize,
        }
    }
}

impl From<(GridSpaceCoordinate, f32)> for Vec3 {
    fn from(val: (GridSpaceCoordinate, f32)) -> Self {
        let (coord, scale) = &val;

        Vec3 {
            x: coord.x as f32 * scale,
            y: coord.y as f32 * scale,
            z: coord.z as f32 * scale,
        }
    }
}

impl<T: Bounded> From<(&T, HashGridInsertionStrategy, f32)> for GridSpaceCoordinate {
    fn from(data: (&T, HashGridInsertionStrategy, f32)) -> Self {
        let (bounded, strategy, scale) = data;

        let aabb = bounded.aabb();

        let point_of_interest = match strategy {
            HashGridInsertionStrategy::AABBCenter => aabb.center(),
        };

        Self::from((point_of_interest, scale))
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub enum HashGridInsertionStrategy {
    #[default]
    AABBCenter,
}

#[derive(Debug, Clone)]
pub struct HashGrid {
    pub scale: f32,
    pub map: HashMap<GridSpaceCoordinate, Vec<usize>>,
    pub strategy: HashGridInsertionStrategy,
}

impl Default for HashGrid {
    fn default() -> Self {
        Self {
            strategy: Default::default(),
            scale: 1.0,
            map: Default::default(),
        }
    }
}

impl HashGrid {
    pub fn new(strategy: HashGridInsertionStrategy, scale: f32) -> Self {
        Self {
            strategy,
            scale,
            ..Default::default()
        }
    }

    /// Rebuilds the hash grid by inserting all items from the iterator.
    pub fn rebuild<'a, T: Bounded + 'a>(&mut self, items: impl Iterator<Item = &'a T>) {
        self.map.clear();

        for (i, item) in items.enumerate() {
            let coord = GridSpaceCoordinate::from((item, self.strategy, self.scale));

            match self.map.get_mut(&coord) {
                Some(list) => {
                    list.push(i);
                }
                None => {
                    let mut cell = Vec::<usize>::with_capacity(4);

                    cell.insert(0, i);

                    self.map.insert(coord, cell);
                }
            }
        }
    }

    /// Renders the hash grid visualization as semi-transparent cubes.
    pub fn render(&self, renderer: &mut SoftwareRenderer, color: Option<Color>) {
        for grid_coord in self.map.keys() {
            let transform = {
                let scale = Mat4::scale_uniform(0.5 * self.scale);

                let offset = Mat4::translation(vec3::ONES * 0.5);

                let translate = Mat4::translation(Vec3 {
                    x: (grid_coord.x as f32 * self.scale),
                    y: (grid_coord.y as f32 * self.scale),
                    z: (grid_coord.z as f32 * self.scale),
                });

                scale * offset * translate
            };

            renderer.render_empty(&transform, EmptyDisplayKind::Cube, false, color);
        }
    }
}
