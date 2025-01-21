pub use glam::f32::*;
pub use glam::u32::*;
pub use image::GenericImageView;

use tar::Archive;
pub type MapFile<R> = Archive<R>;

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

pub struct Segment {
    pub start: Vec2,
    pub end: Vec2,
}

impl Segment {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }

    pub fn dx(&self) -> f32 {
        self.end.x - self.start.x
    }

    pub fn dy(&self) -> f32 {
        self.end.y - self.start.y
    }

    pub fn length(&self) -> f32 {
        self.start.distance(self.end)
    }

    pub fn distance(&self, point: Vec2) -> f32 {
        let closest = self.closest_point(point);
        closest.distance(point)
    }

    pub fn closest_point(&self, point: Vec2) -> Vec2 {
        let length = self.length();
        if length == 0.0 {
            return self.start;
        }

        let dir = self.end - self.start;
        let to_point = point - self.start;

        let t = to_point.dot(dir) / dir.dot(dir);

        if t <= 0.0 {
            return self.start;
        } else if t >= 1.0 {
            return self.end;
        } else {
            return self.start + dir * t;
        }
    }

    pub fn interpolate(&self, t: f32) -> Vec2 {
        self.start + (self.end - self.start) * t
    }
}
