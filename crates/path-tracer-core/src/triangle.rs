use crate::aabb::Aabb;
use crate::hittable::{HitRecord, Hittable, Interval};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Point3;

/// A triangle defined by three vertices.
pub struct Triangle {
    pub v0: Point3,
    pub v1: Point3,
    pub v2: Point3,
    pub material: Material,
}

impl Triangle {
    pub fn new(v0: Point3, v1: Point3, v2: Point3, material: Material) -> Self {
        Self {
            v0,
            v1,
            v2,
            material,
        }
    }

    pub fn bounding_box(&self) -> Aabb {
        const EPS: f64 = 1e-4;
        let min = Point3::new(
            self.v0.x.min(self.v1.x).min(self.v2.x) - EPS,
            self.v0.y.min(self.v1.y).min(self.v2.y) - EPS,
            self.v0.z.min(self.v1.z).min(self.v2.z) - EPS,
        );
        let max = Point3::new(
            self.v0.x.max(self.v1.x).max(self.v2.x) + EPS,
            self.v0.y.max(self.v1.y).max(self.v2.y) + EPS,
            self.v0.z.max(self.v1.z).max(self.v2.z) + EPS,
        );
        Aabb::new(min, max)
    }
}

/// Möller-Trumbore ray-triangle intersection.
impl Hittable for Triangle {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = ray.direction.cross(edge2);
        let det = edge1.dot(h);

        // Ray is parallel to the triangle.
        if det.abs() < 1e-10 {
            return None;
        }

        let inv_det = 1.0 / det;
        let s = ray.origin - self.v0;
        let u = inv_det * s.dot(h);
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(edge1);
        let v = inv_det * ray.direction.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = inv_det * edge2.dot(q);
        if !t_range.surrounds(t) {
            return None;
        }

        let point = ray.at(t);
        let outward_normal = edge1.cross(edge2).normalized();
        Some(HitRecord::new(ray, point, outward_normal, t, self.material))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Material;
    use crate::vec3::{Color, Vec3};

    fn test_triangle() -> Triangle {
        let mat = Material::lambertian(Color::new(0.5, 0.5, 0.5));
        Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            mat,
        )
    }

    #[test]
    fn ray_hits_triangle() {
        let tri = test_triangle();
        let ray = Ray::new(
            Point3::new(0.2, 0.2, 1.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        let hit = tri.hit(&ray, Interval::new(0.001, f64::INFINITY));
        assert!(hit.is_some());
        let rec = hit.unwrap();
        assert!((rec.t - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ray_misses_triangle() {
        let tri = test_triangle();
        let ray = Ray::new(
            Point3::new(2.0, 2.0, 1.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        assert!(tri.hit(&ray, Interval::new(0.001, f64::INFINITY)).is_none());
    }

    #[test]
    fn ray_parallel_to_triangle() {
        let tri = test_triangle();
        let ray = Ray::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        );
        assert!(tri.hit(&ray, Interval::new(0.001, f64::INFINITY)).is_none());
    }

    #[test]
    fn hit_outside_t_range() {
        let tri = test_triangle();
        let ray = Ray::new(
            Point3::new(0.2, 0.2, 1.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        // t=1.0 but range ends at 0.5
        assert!(tri.hit(&ray, Interval::new(0.001, 0.5)).is_none());
    }

    #[test]
    fn bounding_box_has_padding() {
        let tri = test_triangle();
        let bb = tri.bounding_box();
        // All vertices have z=0, but bbox should have thickness from epsilon padding.
        assert!(bb.min.z < 0.0);
        assert!(bb.max.z > 0.0);
    }
}
