use rand::Rng;

use crate::camera::{Camera, CameraConfig};
use crate::hittable::{HittableList, Shape};
use crate::material::Material;
use crate::vec3::{Color, Point3, Vec3};

/// Build the classic "random spheres" demo scene.
pub fn random_spheres_scene() -> (HittableList, Camera) {
    let mut world = HittableList::new();
    let mut rng = rand::rng();

    // Ground
    let ground = Material::lambertian(Color::new(0.5, 0.5, 0.5));
    world.add(Shape::sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground));

    // Small random spheres
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat: f64 = rng.random();
            let center = Point3::new(
                a as f64 + 0.9 * rng.random::<f64>(),
                0.2,
                b as f64 + 0.9 * rng.random::<f64>(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() <= 0.9 {
                continue;
            }

            let material = if choose_mat < 0.8 {
                let albedo = Color::random(&mut rng) * Color::random(&mut rng);
                Material::lambertian(albedo)
            } else if choose_mat < 0.95 {
                let albedo = Color::random_range(&mut rng, 0.5, 1.0);
                let fuzz = rng.random_range(0.0..0.5);
                Material::metal(albedo, fuzz)
            } else {
                Material::dielectric(1.5)
            };

            world.add(Shape::sphere(center, 0.2, material));
        }
    }

    // Three large showcase spheres
    world.add(Shape::sphere(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        Material::dielectric(1.5),
    ));
    world.add(Shape::sphere(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Material::lambertian(Color::new(0.4, 0.2, 0.1)),
    ));
    world.add(Shape::sphere(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Material::metal(Color::new(0.7, 0.6, 0.5), 0.0),
    ));

    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let camera = Camera::new(CameraConfig {
        lookfrom,
        lookat,
        vup: Vec3::new(0.0, 1.0, 0.0),
        vfov: 20.0,
        aspect_ratio: 3.0 / 2.0,
        aperture: 0.1,
        focus_dist: 10.0,
    });

    (world, camera)
}
