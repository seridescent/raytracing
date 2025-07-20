use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::StdRng;
use raytracing::camera::Camera;
use raytracing::hittable::Hittable;
use raytracing::sphere::Sphere;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();

    // World

    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5)?),
        Box::new(Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0)?),
    ];

    let camera = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,
    };

    let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(12)));

    camera.initialize().render(rng, &world);

    eprintln!(
        "\n\nDone! Ran for {:#?}",
        Instant::now().duration_since(start_time)
    );

    Ok(())
}
