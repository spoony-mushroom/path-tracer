use rand::Rng;

use crate::bvh::Bvh;
use crate::camera::{Camera, CameraConfig};
use crate::hittable::Shape;
use crate::material::Material;
use crate::obj;
use crate::vec3::{Color, Point3, Vec3};

/// Build the classic "random spheres" demo scene.
pub fn random_spheres_scene() -> (Bvh, Camera) {
    let mut shapes = Vec::new();
    let mut rng = rand::rng();

    // Ground
    let ground = Material::lambertian(Color::new(0.5, 0.5, 0.5));
    shapes.push(Shape::sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground));

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

            shapes.push(Shape::sphere(center, 0.2, material));
        }
    }

    // Three large showcase spheres
    shapes.push(Shape::sphere(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        Material::dielectric(1.5),
    ));
    shapes.push(Shape::sphere(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Material::lambertian(Color::new(0.4, 0.2, 0.1)),
    ));
    shapes.push(Shape::sphere(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Material::metal(Color::new(0.7, 0.6, 0.5), 0.0),
    ));

    let bvh = Bvh::build(shapes);

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

    (bvh, camera)
}

/// Build a demo scene with a triangle mesh cube alongside spheres.
pub fn mesh_demo_scene() -> (Bvh, Camera) {
    let mut shapes = Vec::new();

    // Ground
    let ground = Material::lambertian(Color::new(0.5, 0.5, 0.5));
    shapes.push(Shape::sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground));

    // Procedural cube made of 12 triangles, centered at origin, side length 2.
    let cube_mat = Material::lambertian(Color::new(0.8, 0.2, 0.2));
    shapes.extend(cube_triangles(Point3::new(0.0, 1.0, 0.0), 1.0, cube_mat));

    // Glass sphere
    shapes.push(Shape::sphere(
        Point3::new(2.5, 1.0, 0.0),
        1.0,
        Material::dielectric(1.5),
    ));

    // Metal sphere
    shapes.push(Shape::sphere(
        Point3::new(-2.5, 1.0, 0.0),
        1.0,
        Material::metal(Color::new(0.7, 0.6, 0.5), 0.0),
    ));

    let bvh = Bvh::build(shapes);

    let lookfrom = Point3::new(6.0, 3.0, 6.0);
    let lookat = Point3::new(0.0, 1.0, 0.0);
    let camera = Camera::new(CameraConfig {
        lookfrom,
        lookat,
        vup: Vec3::new(0.0, 1.0, 0.0),
        vfov: 30.0,
        aspect_ratio: 3.0 / 2.0,
        aperture: 0.05,
        focus_dist: (lookfrom - lookat).length(),
    });

    (bvh, camera)
}

/// Build a scene from an OBJ file.
///
/// The mesh is auto-centered and scaled to fit within a 2-unit bounding sphere,
/// placed on a ground plane with glass and metal spheres for light interactions.
pub fn obj_scene(path: &str) -> (Bvh, Camera) {
    let mesh_mat = Material::lambertian(Color::new(0.8, 0.6, 0.4));
    let mesh_shapes = obj::load_obj(path, mesh_mat)
        .unwrap_or_else(|e| panic!("Failed to load OBJ file '{}': {}", path, e));

    eprintln!("Loaded {} triangles from {}", mesh_shapes.len(), path);

    // Compute bounding box to auto-center and scale the mesh.
    let (min, max) = mesh_bounds(&mesh_shapes);
    let center = (min + max) * 0.5;
    let extent = max - min;
    let max_dim = extent.x.max(extent.y).max(extent.z);
    let scale = if max_dim > 0.0 { 2.0 / max_dim } else { 1.0 };
    let y_offset = extent.y * 0.5 * scale;

    let mut shapes = Vec::with_capacity(mesh_shapes.len() + 3);

    // Ground
    let ground = Material::lambertian(Color::new(0.5, 0.5, 0.5));
    shapes.push(Shape::sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground));

    // Re-center, scale, and shift mesh so its bottom sits on y=0.
    let offset = Vec3::new(0.0, y_offset, 0.0);
    for shape in &mesh_shapes {
        if let Shape::Triangle(tri) = shape {
            let v0 = (tri.v0 - center) * scale + offset;
            let v1 = (tri.v1 - center) * scale + offset;
            let v2 = (tri.v2 - center) * scale + offset;
            shapes.push(Shape::triangle(v0, v1, v2, tri.material));
        }
    }

    // Glass sphere to the right — slightly occluding the mesh.
    shapes.push(Shape::sphere(
        Point3::new(1.8, 1.0, 0.5),
        1.0,
        Material::dielectric(1.1),
    ));

    // Metal sphere to the left — reflects the mesh.
    shapes.push(Shape::sphere(
        Point3::new(-2.5, 1.0, 0.0),
        1.0,
        Material::metal(Color::new(0.7, 0.6, 0.5), 0.0),
    ));

    let bvh = Bvh::build(shapes);

    let lookfrom = Point3::new(5.0, 3.0, 5.0);
    let lookat = Point3::new(0.0, 1.0, 0.0);
    let camera = Camera::new(CameraConfig {
        lookfrom,
        lookat,
        vup: Vec3::new(0.0, 1.0, 0.0),
        vfov: 30.0,
        aspect_ratio: 3.0 / 2.0,
        aperture: 0.05,
        focus_dist: (lookfrom - lookat).length(),
    });

    (bvh, camera)
}

/// Compute the axis-aligned bounding box of a set of triangle shapes.
fn mesh_bounds(shapes: &[Shape]) -> (Point3, Point3) {
    let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
    let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for shape in shapes {
        if let Shape::Triangle(tri) = shape {
            for v in [tri.v0, tri.v1, tri.v2] {
                min.x = min.x.min(v.x);
                min.y = min.y.min(v.y);
                min.z = min.z.min(v.z);
                max.x = max.x.max(v.x);
                max.y = max.y.max(v.y);
                max.z = max.z.max(v.z);
            }
        }
    }
    (min, max)
}

/// Generate the 12 triangles of an axis-aligned cube.
fn cube_triangles(center: Point3, half: f64, material: Material) -> Vec<Shape> {
    let c = center;
    let h = half;
    // 8 vertices of the cube
    let v = [
        Point3::new(c.x - h, c.y - h, c.z - h), // 0: left-bottom-back
        Point3::new(c.x + h, c.y - h, c.z - h), // 1: right-bottom-back
        Point3::new(c.x + h, c.y + h, c.z - h), // 2: right-top-back
        Point3::new(c.x - h, c.y + h, c.z - h), // 3: left-top-back
        Point3::new(c.x - h, c.y - h, c.z + h), // 4: left-bottom-front
        Point3::new(c.x + h, c.y - h, c.z + h), // 5: right-bottom-front
        Point3::new(c.x + h, c.y + h, c.z + h), // 6: right-top-front
        Point3::new(c.x - h, c.y + h, c.z + h), // 7: left-top-front
    ];
    // 6 faces × 2 triangles each, wound counter-clockwise (outward normals)
    let faces: [(usize, usize, usize); 12] = [
        // Front (+Z)
        (4, 5, 6), (4, 6, 7),
        // Back (-Z)
        (1, 0, 3), (1, 3, 2),
        // Right (+X)
        (5, 1, 2), (5, 2, 6),
        // Left (-X)
        (0, 4, 7), (0, 7, 3),
        // Top (+Y)
        (7, 6, 2), (7, 2, 3),
        // Bottom (-Y)
        (0, 1, 5), (0, 5, 4),
    ];
    faces
        .iter()
        .map(|&(a, b, c)| Shape::triangle(v[a], v[b], v[c], material))
        .collect()
}
