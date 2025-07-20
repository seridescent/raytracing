use std::error::Error;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use raytracing::camera::Camera;
use raytracing::hittable::Hittable;
use raytracing::sphere::Sphere;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    // World

    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5)?),
        Box::new(Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0)?),
    ];

    let camera = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 100,
    };

    let mut rng = ChaCha8Rng::seed_from_u64(0);

    camera.initialize().render(&mut rng, &world);

    Ok(())
}
