use std::error::Error;

use raytracing::camera::Camera;
use raytracing::geometry::{ConstructSphereError, Geometry};
use raytracing::material::Material;
use raytracing::runner::RenderRunner;
use raytracing::surface::Surface;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let world = simple_scene()?;

    let camera = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,

        look_from: Vector3::new(0.0, 0.0, 0.0),
        look_at: Vector3::new(0.0, 0.0, -1.0),
        v_fov: 90.0,

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

fn simple_scene() -> Result<Box<[Surface]>, ConstructSphereError> {
    let red_material = Material::Lambertian {
        albedo: Vector3::new(0.7, 0.3, 0.3),
    };
    let blue_material = Material::Lambertian {
        albedo: Vector3::new(0.3, 0.3, 0.7),
    };
    let metal_material = Material::Metal {
        albedo: Vector3::new(0.8, 0.8, 0.9),
        fuzz_radius: 0.0,
    };

    Ok(Box::from([
        Surface::new(
            Geometry::sphere(Vector3::new(0.0, 0.0, -1.0), 0.5)?,
            red_material,
        ),
        Surface::new(
            Geometry::sphere(Vector3::new(-1.0, 0.0, -1.0), 0.5)?,
            blue_material,
        ),
        Surface::new(
            Geometry::sphere(Vector3::new(1.0, 0.0, -1.0), 0.5)?,
            metal_material,
        ),
        Surface::new(
            Geometry::sphere(Vector3::new(0.0, -100.5, -1.0), 100.0)?,
            Material::Lambertian {
                albedo: Vector3::new(0.8, 0.8, 0.0),
            },
        ),
    ]))
}
