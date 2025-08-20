use std::error::Error;

use raytracing::camera::Camera;
use raytracing::geometry::Geometry;
use raytracing::material::Material;
use raytracing::runner::RenderRunner;
use raytracing::surface::Surface;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let world = quads();

    let camera = Camera {
        aspect_ratio: 1.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,

        v_fov: 80.0,
        look_from: Vector3::new(0.0, 0.0, 9.0),
        look_at: Vector3::new(0.0, 0.0, 0.0),
        v_up: Vector3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.0,

        background: Vector3::new(0.7, 0.8, 1.0),

        ..Default::default()
    };

    RenderRunner {
        camera,
        ..Default::default()
    }
    .run(world)
}

fn quads() -> Box<[Surface]> {
    let left_red = Material::Lambertian {
        albedo: Vector3::new(1.0, 0.2, 0.2),
    };
    let back_green = Material::Lambertian {
        albedo: Vector3::new(0.2, 1.0, 0.2),
    };
    let right_blue = Material::Lambertian {
        albedo: Vector3::new(0.2, 0.2, 1.0),
    };
    let upper_orange = Material::Lambertian {
        albedo: Vector3::new(1.0, 0.5, 0.0),
    };
    let lower_teal = Material::Lambertian {
        albedo: Vector3::new(0.2, 0.8, 0.8),
    };

    // Quads
    Box::from([
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(-3.0, -2.0, 5.0),
                Vector3::new(0.0, 0.0, -4.0),
                Vector3::new(0.0, 4.0, 0.0),
            ),
            left_red,
        ),
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(-2.0, -2.0, 0.0),
                Vector3::new(4.0, 0.0, 0.0),
                Vector3::new(0.0, 4.0, 0.0),
            ),
            back_green,
        ),
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(3.0, -2.0, 1.0),
                Vector3::new(0.0, 0.0, 4.0),
                Vector3::new(0.0, 4.0, 0.0),
            ),
            right_blue,
        ),
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(-2.0, 3.0, 1.0),
                Vector3::new(4.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 4.0),
            ),
            upper_orange,
        ),
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(-2.0, -3.0, 5.0),
                Vector3::new(4.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, -4.0),
            ),
            lower_teal,
        ),
    ])
}
