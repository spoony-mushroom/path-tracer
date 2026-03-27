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

/// Render the full image at once, returning an `RgbImage`.
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
    let width = config.image_width;
    let height = config.image_height;
    let spp = config.samples_per_pixel;
    let max_depth = config.max_depth;

    let rows: Vec<Vec<[u8; 3]>> = (0..height)
        .into_par_iter()
        .rev()
        .map(|y| {
            let mut rng = rand::rng();
            let mut row = Vec::with_capacity(width as usize);
            for x in 0..width {
                let mut pixel_color = Color::ZERO;
                for _ in 0..spp {
                    pixel_color += sample_pixel(x, y, width, height, max_depth, camera, world, &mut rng);
                }
                row.push(color_to_rgb(pixel_color, spp));
            }
            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
            on_progress(done, height);
            row
        })
        .collect();

    let mut img = RgbImage::new(width, height);
    for (row_idx, row) in rows.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            img.put_pixel(x as u32, row_idx as u32, image::Rgb(pixel));
        }
    }
    img
}

/// Progressive renderer that accumulates samples incrementally.
///
/// The client drives the rendering loop by calling `refine()` repeatedly,
/// and can stop at any time when the quality is sufficient.
pub struct ProgressiveRenderer {
    width: u32,
    height: u32,
    max_depth: u32,
    accumulator: Vec<Color>,
    sample_count: u32,
}

impl ProgressiveRenderer {
    pub fn new(config: &RenderConfig) -> Self {
        let pixel_count = (config.image_width * config.image_height) as usize;
        Self {
            width: config.image_width,
            height: config.image_height,
            max_depth: config.max_depth,
            accumulator: vec![Color::ZERO; pixel_count],
            sample_count: 0,
        }
    }

    /// Add one sample per pixel (parallelized across rows).
    /// Returns the total number of samples accumulated so far.
    pub fn refine(&mut self, camera: &Camera, world: &(impl Hittable + Sync)) -> u32 {
        let width = self.width;
        let height = self.height;
        let max_depth = self.max_depth;

        let new_samples: Vec<Color> = (0..height)
            .into_par_iter()
            .rev()
            .flat_map(|y| {
                let mut rng = rand::rng();
                (0..width)
                    .map(|x| sample_pixel(x, y, width, height, max_depth, camera, world, &mut rng))
                    .collect::<Vec<_>>()
            })
            .collect();

        for (acc, sample) in self.accumulator.iter_mut().zip(new_samples.iter()) {
            *acc += *sample;
        }
        self.sample_count += 1;
        self.sample_count
    }

    /// Current number of accumulated samples per pixel.
    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    /// Convert the current accumulator to a packed `0x00RRGGBB` buffer
    /// suitable for framebuffer display (e.g., `minifb`).
    pub fn buffer_u32(&self) -> Vec<u32> {
        self.accumulator
            .iter()
            .map(|color| {
                let [r, g, b] = color_to_rgb(*color, self.sample_count);
                (r as u32) << 16 | (g as u32) << 8 | (b as u32)
            })
            .collect()
    }

    /// Convert the current accumulator state to an `RgbImage`.
    pub fn image(&self) -> RgbImage {
        let mut img = RgbImage::new(self.width, self.height);
        for (i, color) in self.accumulator.iter().enumerate() {
            let x = (i as u32) % self.width;
            let y = (i as u32) / self.width;
            img.put_pixel(x, y, image::Rgb(color_to_rgb(*color, self.sample_count)));
        }
        img
    }
}

/// Trace a single sample for one pixel.
fn sample_pixel(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    max_depth: u32,
    camera: &Camera,
    world: &impl Hittable,
    rng: &mut impl Rng,
) -> Color {
    let u = (x as f64 + rng.random::<f64>()) / (width - 1) as f64;
    let v = (y as f64 + rng.random::<f64>()) / (height - 1) as f64;
    let ray = camera.get_ray(u, v, rng);
    ray_color(&ray, world, max_depth, rng)
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
