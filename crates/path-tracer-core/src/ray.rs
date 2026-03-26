use crate::vec3::{Point3, Vec3};

/// A ray defined by an origin point and a direction.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub const fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Compute the point at parameter `t` along the ray.
    pub fn at(self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }
}
