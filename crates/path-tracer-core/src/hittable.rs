use crate::aabb::Aabb;
use crate::material::Material;
use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::vec3::{Point3, Vec3};

/// Record of a ray-surface intersection.
pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub material: Material,
}

impl HitRecord {
    /// Construct a hit record with the outward-facing normal convention.
    /// `outward_normal` must be a unit vector.
    pub fn new(
        ray: &Ray,
        point: Point3,
        outward_normal: Vec3,
        t: f64,
        material: Material,
    ) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self {
            point,
            normal,
            t,
            front_face,
            material,
        }
    }
}

/// An interval on the real line, used to constrain ray `t` values.
#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    #[allow(dead_code)]
    pub fn contains(self, t: f64) -> bool {
        self.min <= t && t <= self.max
    }

    pub fn surrounds(self, t: f64) -> bool {
        self.min < t && t < self.max
    }
}

/// Trait for objects that can be intersected by a ray.
pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord>;
}

/// Concrete shape type using enum dispatch for cache-friendly intersection testing.
pub enum Shape {
    Sphere(Sphere),
}

impl Shape {
    pub fn sphere(center: Point3, radius: f64, material: Material) -> Self {
        Self::Sphere(Sphere::new(center, radius, material))
    }

    pub fn bounding_box(&self) -> Aabb {
        match self {
            Self::Sphere(s) => s.bounding_box(),
        }
    }
}

impl Hittable for Shape {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        match self {
            Self::Sphere(s) => s.hit(ray, t_range),
        }
    }
}

/// A collection of shapes stored contiguously for cache-friendly iteration.
pub struct HittableList {
    objects: Vec<Shape>,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: Shape) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        let mut closest = t_range.max;
        let mut result = None;

        for object in &self.objects {
            if let Some(record) = object.hit(ray, Interval::new(t_range.min, closest)) {
                closest = record.t;
                result = Some(record);
            }
        }

        result
    }
}
