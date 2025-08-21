use std::error::Error;

use raytracing::camera::Camera;
use raytracing::geometry::Geometry;
use raytracing::material::Material;
use raytracing::runner::RenderRunner;
use raytracing::surface::Surface;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let world = scene();

    let camera = Camera {
        aspect_ratio: 1.0,
        image_width: 600,
        samples_per_pixel: 2000,
        max_depth: 50,

        v_fov: 40.0,
        look_from: Vector3::new(278.0, 278.0, -800.0),
        look_at: Vector3::new(278.0, 278.0, 0.0),
        v_up: Vector3::new(0.0, 1.0, 0.0),

        background: Vector3::new(0.0, 0.0, 0.0),

        ..Default::default()
    };

    RenderRunner {
        camera,
        ..Default::default()
    }
    .run(world)
}

fn scene() -> Box<[Surface]> {
    let white = Material::Lambertian {
        albedo: Vector3::new(0.73, 0.73, 0.73),
    };

    let mut surfaces = Vec::new();

    // First box: rotated 15 degrees, then translated by (265, 0, 295)
    surfaces.extend(box_geometry(
        Vector3::new(0.0, 0.0, 0.0) + Vector3::new(265.0, 0.0, 295.0),
        Vector3::new(165.0, 330.0, 165.0) + Vector3::new(265.0, 0.0, 295.0),
        Material::Metal {
            albedo: Vector3::new(0.7, 0.6, 0.5),
            fuzz_radius: 0.0,
        },
        18.0_f64.to_radians(),
    ));

    // Second box: rotated -18 degrees, then translated by (130, 0, 65)
    surfaces.extend(box_geometry(
        Vector3::new(0.0, 0.0, 0.0) + Vector3::new(100.0, 0.0, 65.0),
        Vector3::new(165.0, 165.0, 165.0) + Vector3::new(100.0, 0.0, 65.0),
        white,
        (-18.0_f64).to_radians(),
    ));

    surfaces.extend(cornell_box());

    surfaces.into_boxed_slice()
}

fn box_geometry(a: Vector3, b: Vector3, material: Material, theta: f64) -> Box<[Surface]> {
    let min = Vector3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    let max = Vector3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

    // Calculate center for rotation
    let center = min + (max - min) * 0.5;

    // Rotation around Y-axis
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();

    let rotate_y = |v: Vector3| -> Vector3 {
        let relative = v - center;
        let rotated = Vector3::new(
            cos_theta * relative.x + sin_theta * relative.z,
            relative.y,
            -sin_theta * relative.x + cos_theta * relative.z,
        );
        rotated + center
    };

    // Define all 8 vertices of the box
    let v000 = rotate_y(Vector3::new(min.x, min.y, min.z));
    let v001 = rotate_y(Vector3::new(min.x, min.y, max.z));
    let v010 = rotate_y(Vector3::new(min.x, max.y, min.z));
    let v011 = rotate_y(Vector3::new(min.x, max.y, max.z));
    let v100 = rotate_y(Vector3::new(max.x, min.y, min.z));
    let v101 = rotate_y(Vector3::new(max.x, min.y, max.z));
    let v110 = rotate_y(Vector3::new(max.x, max.y, min.z));
    let v111 = rotate_y(Vector3::new(max.x, max.y, max.z));

    Box::from([
        // front face (z = max.z): v001 -> v101 -> v111 -> v011
        Surface::new(
            Geometry::quadrilateral(v001, v101 - v001, v011 - v001),
            material.clone(),
        ),
        // back face (z = min.z): v100 -> v000 -> v010 -> v110
        Surface::new(
            Geometry::quadrilateral(v100, v000 - v100, v110 - v100),
            material.clone(),
        ),
        // left face (x = min.x): v000 -> v001 -> v011 -> v010
        Surface::new(
            Geometry::quadrilateral(v000, v001 - v000, v010 - v000),
            material.clone(),
        ),
        // right face (x = max.x): v101 -> v100 -> v110 -> v111
        Surface::new(
            Geometry::quadrilateral(v101, v100 - v101, v111 - v101),
            material.clone(),
        ),
        // bottom face (y = min.y): v000 -> v100 -> v101 -> v001
        Surface::new(
            Geometry::quadrilateral(v000, v100 - v000, v001 - v000),
            material.clone(),
        ),
        // top face (y = max.y): v010 -> v011 -> v111 -> v110
        Surface::new(
            Geometry::quadrilateral(v010, v011 - v010, v110 - v010),
            material,
        ),
    ])
}

fn cornell_box() -> Box<[Surface]> {
    let red = Material::Lambertian {
        albedo: Vector3::new(0.65, 0.05, 0.05),
    };
    let white = Material::Lambertian {
        albedo: Vector3::new(0.73, 0.73, 0.73),
    };
    let green = Material::Lambertian {
        albedo: Vector3::new(0.12, 0.45, 0.15),
    };
    let light = Material::DiffuseLight {
        emit: Vector3::new(50.0, 50.0, 50.0),
    };

    Box::from([
        // Right wall (green)
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(555.0, 0.0, 0.0),
                Vector3::new(0.0, 555.0, 0.0),
                Vector3::new(0.0, 0.0, 555.0),
            ),
            red,
        ),
        // Left wall (red)
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 555.0, 0.0),
                Vector3::new(0.0, 0.0, 555.0),
            ),
            green,
        ),
        // Light
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(343.0, 554.0, 332.0),
                Vector3::new(-130.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, -105.0),
            ),
            light,
        ),
        // Floor
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(555.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 555.0),
            ),
            white.clone(),
        ),
        // Ceiling
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(555.0, 555.0, 555.0),
                Vector3::new(-555.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, -555.0),
            ),
            white.clone(),
        ),
        // Back wall
        Surface::new(
            Geometry::quadrilateral(
                Vector3::new(0.0, 0.0, 555.0),
                Vector3::new(555.0, 0.0, 0.0),
                Vector3::new(0.0, 555.0, 0.0),
            ),
            white.clone(),
        ),
    ])
}
