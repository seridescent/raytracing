use std::error::Error;

use raytracing::camera::Camera;
use raytracing::geometry::{ConstructSphereError, Geometry};
use raytracing::material::Material;
use raytracing::runner::RenderRunner;
use raytracing::surface::Surface;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let world = demo_spheres()?;

    let camera = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,

        look_from: Vector3::new(-2.0, 2.0, 1.0),
        look_at: Vector3::new(0.0, 0.0, -1.0),
        v_fov: 20.0,

        defocus_angle: 10.0,
        focus_dist: 3.4,

        ..Default::default()
    };

    // FIXME: concentric spheres currently break partitioning
    RenderRunner {
        camera,
        ..Default::default()
    }
    .run(world)
}

fn demo_spheres() -> Result<Box<[Surface]>, ConstructSphereError> {
    let material_ground = Material::Lambertian {
        albedo: Vector3::new(0.8, 0.8, 0.0),
    };
    let material_center = Material::Lambertian {
        albedo: Vector3::new(0.1, 0.2, 0.5),
    };
    let material_left = Material::Dielectric {
        refraction_index: 1.5,
    };
    let material_bubble = Material::Dielectric {
        refraction_index: 1.0 / 1.5,
    };
    let material_right = Material::Metal {
        albedo: Vector3::new(0.8, 0.6, 0.2),
        fuzz_radius: 1.0,
    };

    Ok(Box::from([
        Surface::new(
            Geometry::sphere(Vector3::new(0.0, -100.5, -1.0), 100.0)?,
            material_ground,
        ),
        Surface::new(
            Geometry::sphere(Vector3::new(0.0, 0.0, -1.2), 0.5)?,
            material_center,
        ),
        Surface::new(
            Geometry::sphere(Vector3::new(-1.0, 0.0, -1.0), 0.5)?,
            material_left,
        ),
        Surface::new(
            Geometry::sphere(Vector3::new(-1.0, 0.0, -1.0), 0.4)?,
            material_bubble,
        ),
        Surface::new(
            Geometry::sphere(Vector3::new(1.0, 0.0, -1.0), 0.5)?,
            material_right,
        ),
    ]))
}
