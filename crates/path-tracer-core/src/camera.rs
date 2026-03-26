use rand::Rng;

use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// Camera configuration.
pub struct CameraConfig {
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub vup: Vec3,
    /// Vertical field of view in degrees.
    pub vfov: f64,
    pub aspect_ratio: f64,
    /// Aperture diameter (0 = pinhole, no depth of field).
    pub aperture: f64,
    /// Distance to the focus plane.
    pub focus_dist: f64,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            lookfrom: Point3::new(0.0, 0.0, 0.0),
            lookat: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            vfov: 90.0,
            aspect_ratio: 16.0 / 9.0,
            aperture: 0.0,
            focus_dist: 1.0,
        }
    }
}

/// A camera that generates rays for path tracing.
pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: f64,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        let theta = config.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = config.aspect_ratio * viewport_height;

        let w = (config.lookfrom - config.lookat).normalized();
        let u = config.vup.cross(w).normalized();
        let v = w.cross(u);

        let horizontal = u * viewport_width * config.focus_dist;
        let vertical = v * viewport_height * config.focus_dist;
        let lower_left_corner =
            config.lookfrom - horizontal / 2.0 - vertical / 2.0 - w * config.focus_dist;

        Self {
            origin: config.lookfrom,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius: config.aperture / 2.0,
        }
    }

    /// Generate a ray for the given (s, t) viewport coordinates in [0, 1].
    pub fn get_ray(&self, s: f64, t: f64, rng: &mut impl Rng) -> Ray {
        let rd = Vec3::random_in_unit_disk(rng) * self.lens_radius;
        let offset = self.u * rd.x + self.v * rd.y;

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + self.horizontal * s + self.vertical * t
                - self.origin
                - offset,
        )
    }
}
