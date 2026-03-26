use std::time::Instant;

use path_tracer_core::render::{ProgressiveRenderer, RenderConfig};
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
        "Rendering {}x{} @ {} spp (progressive)...",
        config.image_width, config.image_height, config.samples_per_pixel
    );
    let start = Instant::now();

    let mut renderer = ProgressiveRenderer::new(&config);

    loop {
        let samples = renderer.refine(&camera, &world);

        if samples % 10 == 0 || samples >= config.samples_per_pixel {
            let img = renderer.image();
            let path = format!("output_{samples:03}.png");
            img.save(&path).expect("Failed to write output image");
            let elapsed = start.elapsed();
            eprintln!("  pass {samples}/{} -> {path} ({elapsed:.2?})", config.samples_per_pixel);
        }

        if samples >= config.samples_per_pixel {
            break;
        }
    }
}
