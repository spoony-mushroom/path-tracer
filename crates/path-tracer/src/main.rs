use std::time::Instant;

use path_tracer_core::render::{RenderConfig, render_image};
use path_tracer_core::scene;

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

    let img = render_image(&config, &camera, &world, |done, total| {
        if done % 50 == 0 || done == total {
            eprintln!("  rows: {done}/{total}");
        }
    });

    let elapsed = start.elapsed();
    eprintln!("Render complete in {elapsed:.2?}");

    let output_path = "output.png";
    img.save(output_path).expect("Failed to write output image");
    eprintln!("Saved to {output_path}");
}
