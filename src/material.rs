use rand::Rng;

use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec3::{Color, Vec3};

/// Result of scattering a ray off a surface.
pub struct ScatterResult {
    pub attenuation: Color,
    pub scattered: Ray,
}

/// Material types using an enum for cache-friendly dispatch.
#[derive(Debug, Clone, Copy)]
pub enum Material {
    /// Ideal diffuse (Lambertian) surface.
    Lambertian { albedo: Color },

    /// Metallic surface with optional fuzz (0 = perfect mirror).
    Metal { albedo: Color, fuzz: f64 },

    /// Dielectric (glass) with a given index of refraction.
    Dielectric { refraction_index: f64 },
}

impl Material {
    pub fn lambertian(albedo: Color) -> Self {
        Self::Lambertian { albedo }
    }

    pub fn metal(albedo: Color, fuzz: f64) -> Self {
        Self::Metal {
            albedo,
            fuzz: fuzz.min(1.0),
        }
    }

    pub fn dielectric(refraction_index: f64) -> Self {
        Self::Dielectric { refraction_index }
    }

    /// Scatter an incoming ray according to this material's properties.
    pub fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut impl Rng) -> Option<ScatterResult> {
        match *self {
            Self::Lambertian { albedo } => scatter_lambertian(albedo, hit, rng),
            Self::Metal { albedo, fuzz } => scatter_metal(albedo, fuzz, ray, hit, rng),
            Self::Dielectric { refraction_index } => {
                scatter_dielectric(refraction_index, ray, hit, rng)
            }
        }
    }
}

fn scatter_lambertian(
    albedo: Color,
    hit: &HitRecord,
    rng: &mut impl Rng,
) -> Option<ScatterResult> {
    let mut scatter_dir = hit.normal + Vec3::random_unit_vector(rng);
    if scatter_dir.near_zero() {
        scatter_dir = hit.normal;
    }
    Some(ScatterResult {
        attenuation: albedo,
        scattered: Ray::new(hit.point, scatter_dir),
    })
}

fn scatter_metal(
    albedo: Color,
    fuzz: f64,
    ray: &Ray,
    hit: &HitRecord,
    rng: &mut impl Rng,
) -> Option<ScatterResult> {
    let reflected = ray.direction.normalized().reflect(hit.normal);
    let scattered = Ray::new(
        hit.point,
        reflected + Vec3::random_unit_vector(rng) * fuzz,
    );
    if scattered.direction.dot(hit.normal) > 0.0 {
        Some(ScatterResult {
            attenuation: albedo,
            scattered,
        })
    } else {
        None
    }
}

fn scatter_dielectric(
    refraction_index: f64,
    ray: &Ray,
    hit: &HitRecord,
    rng: &mut impl Rng,
) -> Option<ScatterResult> {
    let ri = if hit.front_face {
        1.0 / refraction_index
    } else {
        refraction_index
    };

    let unit_direction = ray.direction.normalized();
    let cos_theta = (-unit_direction).dot(hit.normal).min(1.0);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let cannot_refract = ri * sin_theta > 1.0;
    let direction = if cannot_refract || reflectance(cos_theta, ri) > rng.random::<f64>() {
        unit_direction.reflect(hit.normal)
    } else {
        unit_direction.refract(hit.normal, ri)
    };

    Some(ScatterResult {
        attenuation: Color::new(1.0, 1.0, 1.0),
        scattered: Ray::new(hit.point, direction),
    })
}

/// Schlick's approximation for reflectance.
fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
    let r0 = ((1.0 - refraction_index) / (1.0 + refraction_index)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
