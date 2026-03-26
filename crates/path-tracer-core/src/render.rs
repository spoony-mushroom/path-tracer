use std::sync::atomic::{AtomicU32, Ordering};

use image::RgbImage;
use rand::Rng;
use rayon::prelude::*;

use crate::camera::Camera;
use crate::hittable::{Hittable, Interval};
use crate::ray::Ray;
use crate::vec3::Color;

/// Configuration for the renderer.
pub struct RenderConfig {
    pub image_width: u32,
    pub image_height: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
}

/// Render the full image, returning an `RgbImage`.
///
/// Rows are rendered in parallel. The `on_progress` callback is invoked
/// with `(completed_rows, total_rows)` as rendering proceeds.
pub fn render_image(
    config: &RenderConfig,
    camera: &Camera,
    world: &(impl Hittable + Sync),
    on_progress: impl Fn(u32, u32) + Sync,
) -> RgbImage {
    let completed = AtomicU32::new(0);

    let rows: Vec<Vec<[u8; 3]>> = (0..config.image_height)
        .into_par_iter()
        .rev()
        .map(|y| {
            let row = render_row(y, config, camera, world);
            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
            on_progress(done, config.image_height);
            row
        })
        .collect();

    let mut img = RgbImage::new(config.image_width, config.image_height);
    for (row_idx, row) in rows.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            img.put_pixel(x as u32, row_idx as u32, image::Rgb(pixel));
        }
    }
    img
}

/// Trace a single ray through the scene, recursively bouncing off surfaces.
fn ray_color(ray: &Ray, world: &impl Hittable, depth: u32, rng: &mut impl Rng) -> Color {
    if depth == 0 {
        return Color::ZERO;
    }

    // t_min = 0.001 to avoid shadow acne from self-intersection.
    if let Some(hit) = world.hit(ray, Interval::new(0.001, f64::INFINITY)) {
        return match hit.material.scatter(ray, &hit, rng) {
            Some(result) => {
                result.attenuation * ray_color(&result.scattered, world, depth - 1, rng)
            }
            None => Color::ZERO,
        };
    }

    // Sky gradient background.
    let unit_dir = ray.direction.normalized();
    let a = 0.5 * (unit_dir.y + 1.0);
    Color::new(1.0, 1.0, 1.0) * (1.0 - a) + Color::new(0.5, 0.7, 1.0) * a
}

/// Render a single row of pixels, returning a Vec of (r, g, b) bytes.
fn render_row(
    y: u32,
    config: &RenderConfig,
    camera: &Camera,
    world: &(impl Hittable + Sync),
) -> Vec<[u8; 3]> {
    let mut rng = rand::rng();
    let mut row = Vec::with_capacity(config.image_width as usize);

    for x in 0..config.image_width {
        let mut pixel_color = Color::ZERO;

        for _ in 0..config.samples_per_pixel {
            let u = (x as f64 + rng.random::<f64>()) / (config.image_width - 1) as f64;
            let v = (y as f64 + rng.random::<f64>()) / (config.image_height - 1) as f64;
            let ray = camera.get_ray(u, v, &mut rng);
            pixel_color += ray_color(&ray, world, config.max_depth, &mut rng);
        }

        row.push(color_to_rgb(pixel_color, config.samples_per_pixel));
    }

    row
}

/// Convert accumulated color to gamma-corrected 8-bit RGB.
fn color_to_rgb(color: Color, samples: u32) -> [u8; 3] {
    let scale = 1.0 / samples as f64;
    // Gamma 2.0 correction (sqrt).
    let r = (color.x * scale).sqrt().clamp(0.0, 0.999);
    let g = (color.y * scale).sqrt().clamp(0.0, 0.999);
    let b = (color.z * scale).sqrt().clamp(0.0, 0.999);
    [
        (r * 256.0) as u8,
        (g * 256.0) as u8,
        (b * 256.0) as u8,
    ]
}
