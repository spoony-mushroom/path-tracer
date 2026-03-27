use std::time::Instant;

use minifb::{Key, Window, WindowOptions};

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

    let width = config.image_width as usize;
    let height = config.image_height as usize;

    let mut window = Window::new(
        "Path Tracer",
        width,
        height,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let start = Instant::now();
    let mut renderer = ProgressiveRenderer::new(&config);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if renderer.sample_count() < config.samples_per_pixel {
            renderer.refine(&camera, &world);
            let elapsed = start.elapsed();
            eprintln!(
                "  pass {}/{} ({elapsed:.2?})",
                renderer.sample_count(),
                config.samples_per_pixel
            );
        }

        let buffer = renderer.buffer_u32();
        window
            .update_with_buffer(&buffer, width, height)
            .expect("Failed to update window");
    }

    // Save final image on exit.
    let output_path = "output.png";
    renderer
        .image()
        .save(output_path)
        .expect("Failed to write output image");
    eprintln!("Saved to {output_path}");
}
