use crate::hittable::Interval;
use crate::ray::Ray;
use crate::vec3::Point3;

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Point3,
    pub max: Point3,
}

impl Aabb {
    pub const fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    /// Ray-box intersection using the slab method.
    pub fn hit(&self, ray: &Ray, mut t_range: Interval) -> bool {
        for axis in 0..3 {
            let inv_d = 1.0 / ray.direction.axis(axis);
            let mut t0 = (self.min.axis(axis) - ray.origin.axis(axis)) * inv_d;
            let mut t1 = (self.max.axis(axis) - ray.origin.axis(axis)) * inv_d;
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            t_range.min = t_range.min.max(t0);
            t_range.max = t_range.max.min(t1);
            if t_range.max <= t_range.min {
                return false;
            }
        }
        true
    }

    /// Merge two AABBs into one enclosing both.
    pub fn surrounding(a: Self, b: Self) -> Self {
        Self {
            min: Point3::new(
                a.min.x.min(b.min.x),
                a.min.y.min(b.min.y),
                a.min.z.min(b.min.z),
            ),
            max: Point3::new(
                a.max.x.max(b.max.x),
                a.max.y.max(b.max.y),
                a.max.z.max(b.max.z),
            ),
        }
    }

    /// Center of the bounding box.
    pub fn centroid(self) -> Point3 {
        (self.min + self.max) * 0.5
    }

    /// Index of the longest axis (0=X, 1=Y, 2=Z).
    pub fn longest_axis(self) -> usize {
        let extent = self.max - self.min;
        if extent.x > extent.y && extent.x > extent.z {
            0
        } else if extent.y > extent.z {
            1
        } else {
            2
        }
    }
}
