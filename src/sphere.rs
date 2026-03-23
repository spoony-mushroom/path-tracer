use crate::hittable::{HitRecord, Hittable, Interval};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Point3;

/// A sphere defined by its center and radius.
pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();

        // Find the nearest root within the acceptable range.
        let mut root = (h - sqrt_d) / a;
        if !t_range.surrounds(root) {
            root = (h + sqrt_d) / a;
            if !t_range.surrounds(root) {
                return None;
            }
        }

        let point = ray.at(root);
        let outward_normal = (point - self.center) / self.radius;
        Some(HitRecord::new(
            ray,
            point,
            outward_normal,
            root,
            self.material,
        ))
    }
}
