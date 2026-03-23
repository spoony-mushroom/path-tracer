use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use rayon::prelude::*;

use path_tracer::render::{RenderConfig, render_row};
use path_tracer::scene;

fn main() {
    let config = RenderConfig {
        image_width: 600,
        image_height: 400,
        samples_per_pixel: 100,
        max_depth: 50,
    };

    let (world, camera) = scene::random_spheres_scene();

    eprintln!(
        "Rendering {}x{} @ {} spp...",
        config.image_width, config.image_height, config.samples_per_pixel
    );
    let start = Instant::now();

    let completed = AtomicU32::new(0);

    // Render rows in parallel (top to bottom, image origin at bottom-left).
    let rows: Vec<Vec<[u8; 3]>> = (0..config.image_height)
        .into_par_iter()
        .rev()
        .map(|y| {
            let row = render_row(y, &config, &camera, &world);
            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
            if done % 50 == 0 || done == config.image_height {
                eprintln!("  rows: {done}/{}", config.image_height);
            }
            row
        })
        .collect();

    let elapsed = start.elapsed();
    eprintln!("Render complete in {elapsed:.2?}");

    // Write output image.
    let mut img = image::RgbImage::new(config.image_width, config.image_height);
    for (row_idx, row) in rows.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            img.put_pixel(x as u32, row_idx as u32, image::Rgb(pixel));
        }
    }

    let output_path = "output.png";
    img.save(output_path).expect("Failed to write output image");
    eprintln!("Saved to {output_path}");
}
