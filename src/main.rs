use std::error::Error;
use std::f64::consts::PI;
use std::rc::Rc;
use std::time::Instant;

use raytracing::camera::Camera;
use raytracing::hittable::Hittable;
use raytracing::material::Lambertian;
use raytracing::sphere::Sphere;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();

    // World

    let radius = (PI / 4.0).cos();
    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(
            Vector3::new(-radius, 0.0, -1.0),
            radius,
            Rc::new(Lambertian::new(Vector3::new(0.0, 0.0, 1.0))),
        )?),
        Box::new(Sphere::new(
            Vector3::new(radius, 0.0, -1.0),
            radius,
            Rc::new(Lambertian::new(Vector3::new(1.0, 0.0, 0.0))),
        )?),
    ];

    let camera = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,

        ..Default::default()
    };

    camera.initialize().render(&world);

    eprintln!(
        "\n\nDone! Ran for {:#?}",
        Instant::now().duration_since(start_time)
    );

    Ok(())
}
