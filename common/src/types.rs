pub use geo::{prelude::*, Line};
use geo::Closest;
pub use glam::f32::*;
pub use glam::u32::*;

pub fn line_distance(line: &Line<f32>, point: Vec2) -> f32 {
    Euclidean::distance(
        line,
        geo::Coord {
            x: point.x,
            y: point.y,
        },
    )
}

pub fn closest_point_on_line(line: &Line<f32>, point: Vec2) -> Vec2 {
    let closest = line.closest_point(&geo::Point::new(point.x, point.y));
    match closest {
        Closest::Intersection(point) => Vec2::new(point.x(), point.y()),
        Closest::SinglePoint(point) => Vec2::new(point.x(), point.y()),
        Closest::Indeterminate => panic!("indeterminate closest point"),
    }
}

pub type Position = Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Rotation(pub Vec3);
impl Rotation {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }

    pub fn to_rad(&self) -> Vec3 {
        Vec3::new(
            self.x.to_radians(),
            self.y.to_radians(),
            self.z.to_radians(),
        )
    }
}

impl std::ops::Deref for Rotation {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Rotation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::AddAssign<&Rotation> for Rotation {
    fn add_assign(&mut self, rhs: &Rotation) {
        self.0 += rhs.0;
    }
}

impl std::ops::Add<Rotation> for Rotation {
    type Output = Rotation;
    fn add(self, rhs: Rotation) -> Rotation {
        Rotation(self.0 + rhs.0)
    }
}

impl std::ops::SubAssign<&Rotation> for Rotation {
    fn sub_assign(&mut self, rhs: &Rotation) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Sub<Rotation> for Rotation {
    type Output = Rotation;
    fn sub(self, rhs: Rotation) -> Rotation {
        Rotation(self.0 - rhs.0)
    }
}
