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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::Vec3;

    fn unit_box() -> Aabb {
        Aabb::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, 1.0, 1.0))
    }

    #[test]
    fn test_hit() {
        let bbox = unit_box();
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(bbox.hit(&ray, Interval::new(0.001, f64::INFINITY)));
    }

    #[test]
    fn test_miss() {
        let bbox = unit_box();
        let ray = Ray::new(Point3::new(0.0, 5.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(!bbox.hit(&ray, Interval::new(0.001, f64::INFINITY)));
    }

    #[test]
    fn test_hit_negative_direction() {
        let bbox = unit_box();
        let ray = Ray::new(Point3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(bbox.hit(&ray, Interval::new(0.001, f64::INFINITY)));
    }

    #[test]
    fn test_surrounding() {
        let a = Aabb::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let b = Aabb::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(0.5, 0.5, 0.5));
        let merged = Aabb::surrounding(a, b);
        assert_eq!(merged.min, Point3::new(-1.0, -1.0, -1.0));
        assert_eq!(merged.max, Point3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_centroid() {
        let bbox = Aabb::new(Point3::new(0.0, 2.0, 4.0), Point3::new(2.0, 4.0, 6.0));
        assert_eq!(bbox.centroid(), Point3::new(1.0, 3.0, 5.0));
    }

    #[test]
    fn test_longest_axis() {
        let x_long = Aabb::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 1.0, 1.0));
        assert_eq!(x_long.longest_axis(), 0);

        let y_long = Aabb::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 10.0, 1.0));
        assert_eq!(y_long.longest_axis(), 1);

        let z_long = Aabb::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 10.0));
        assert_eq!(z_long.longest_axis(), 2);
    }
}
