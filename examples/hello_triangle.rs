use std::error::Error;

use raytracing::camera::Camera;
use raytracing::geometry::Geometry;
use raytracing::material::Material;
use raytracing::runner::RenderRunner;
use raytracing::surface::Surface;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let world = hello_triangle();

    let camera = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 1000,
        max_depth: 50,

        v_fov: 45.0,
        look_from: Vector3::new(0.0, 0.0, 3.0),
        look_at: Vector3::new(0.0, 0.0, 0.0),
        v_up: Vector3::new(0.0, 1.0, 0.0),

        ..Default::default()
    };

    RenderRunner {
        camera,
        ..Default::default()
    }
    .run(world)
}

fn hello_triangle() -> Box<[Surface]> {
    let uv_gradient = Material::UVGradient { intensity: 1.0 };

    let side_length = 2.0;
    let height = side_length * (3.0_f64).sqrt() / 2.0;

    let top = Vector3::new(0.0, height * 0.5, 0.0);
    let bottom_left = Vector3::new(-side_length * 0.5, -height * 0.5, 0.0);
    let bottom_right = Vector3::new(side_length * 0.5, -height * 0.5, 0.0);

    let u = bottom_right - bottom_left;
    let v = top - bottom_left;

    Box::from([Surface::new(
        Geometry::triangle(bottom_left, u, v),
        uv_gradient,
    )])
}
