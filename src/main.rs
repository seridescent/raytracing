use std::error::Error;
use std::rc::Rc;
use std::time::Instant;

use raytracing::camera::Camera;
use raytracing::hittable::Hittable;
use raytracing::material::{Dielectric, Lambertian, Metal};
use raytracing::sphere::Sphere;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();

    // World

    let material_ground = Rc::new(Lambertian::new(Vector3::new(0.8, 0.8, 0.0)));
    let material_center = Rc::new(Lambertian::new(Vector3::new(0.1, 0.2, 0.5)));
    let material_left = Rc::new(Dielectric::new(1.5));
    let material_bubble = Rc::new(Dielectric::new(1.0 / 1.5));
    let material_right = Rc::new(Metal::new(Vector3::new(0.8, 0.6, 0.2), 1.0));

    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(
            Vector3::new(0.0, -100.5, -1.0),
            100.0,
            material_ground,
        )?),
        Box::new(Sphere::new(
            Vector3::new(0.0, 0.0, -1.2),
            0.5,
            material_center,
        )?),
        Box::new(Sphere::new(
            Vector3::new(-1.0, 0.0, -1.0),
            0.5,
            material_left,
        )?),
        Box::new(Sphere::new(
            Vector3::new(-1.0, 0.0, -1.0),
            0.4,
            material_bubble,
        )?),
        Box::new(Sphere::new(
            Vector3::new(1.0, 0.0, -1.0),
            0.5,
            material_right,
        )?),
    ];

    let camera = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,
    };

    camera.initialize().render(&world);

    eprintln!(
        "\n\nDone! Ran for {:#?}",
        Instant::now().duration_since(start_time)
    );

    Ok(())
}
