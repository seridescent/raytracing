use std::error::Error;

use raytracing::camera::Camera;
use raytracing::geometry::{ConstructSphereError, Geometry};
use raytracing::material::Material;
use raytracing::runner::RenderRunner;
use raytracing::surface::Surface;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let world = simple_light()?;

    let camera = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 1000,
        max_depth: 50,

        v_fov: 20.0,
        look_from: Vector3::new(26.0, 3.0, 6.0),
        look_at: Vector3::new(0.0, 2.0, 0.0),
        v_up: Vector3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.0,
        focus_dist: 1.0,

        ..Default::default()
    };

    RenderRunner {
        camera,
        ..Default::default()
    }
    .run(world)
}

fn simple_light() -> Result<Box<[Surface]>, ConstructSphereError> {
    // Ground sphere with warm beige color
    let ground_material = Material::Lambertian {
        albedo: Vector3::new(0.6, 0.5, 0.4),
    };

    // Small sphere with soft pink color
    let sphere_material = Material::Lambertian {
        albedo: Vector3::new(0.8, 0.4, 0.6),
    };

    let light_material = Material::DiffuseLight {
        emit: Vector3::new(10.0, 10.0, 10.0),
    };

    Ok(Box::from([
        Surface::new(
            Geometry::sphere(Vector3::new(0.0, -1000.0, 0.0), 1000.0)?,
            ground_material,
        ),
        Surface::new(
            Geometry::sphere(Vector3::new(0.0, 2.0, 0.0), 2.0)?,
            sphere_material,
        ),
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(3.0, 1.0, -2.0),
                Vector3::new(2.0, 0.0, 0.0),
                Vector3::new(0.0, 2.0, 0.0),
            ),
            light_material,
        ),
    ]))
}
