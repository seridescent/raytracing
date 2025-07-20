use std::error::Error;

use raytracing::camera::Camera;
use raytracing::hittable::Hittable;
use raytracing::sphere::Sphere;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    // World

    let world = {
        let mut w: Vec<Box<dyn Hittable>> = Vec::new();

        w.push(Box::new(Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5)?));
        w.push(Box::new(Sphere::new(
            Vector3::new(0.0, -100.5, -1.0),
            100.0,
        )?));

        w
    };

    let camera = Camera::new(16.0 / 9.0, 400);

    camera.initialize().render(&world);

    Ok(())
}
