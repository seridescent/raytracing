use std::error::Error;

use rand::{random, random_range};
use raytracing::camera::Camera;
use raytracing::geometry::{ConstructSphereError, Geometry};
use raytracing::interval::Interval;
use raytracing::material::Material;
use raytracing::runner::RenderRunner;
use raytracing::surface::Surface;
use raytracing::vector::Vector3;

fn main() -> Result<(), Box<dyn Error>> {
    let world = cover_spheres()?;

    let camera = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 1200,
        samples_per_pixel: 500,
        max_depth: 50,

        look_from: Vector3::new(13.0, 2.0, 3.0),
        look_at: Vector3::new(0.0, 0.0, 0.0),
        v_fov: 20.0,

        defocus_angle: 0.6,
        focus_dist: 10.0,

        background: Vector3::new(0.7, 0.8, 1.0),

        ..Default::default()
    };

    RenderRunner {
        camera,
        ..Default::default()
    }
    .run(world)
}

fn cover_spheres() -> Result<Box<[Surface]>, ConstructSphereError> {
    const SMALL_SPHERES_RADIUS: f64 = 0.2;
    const BIG_SPHERES_RADIUS: f64 = 1.0;

    let ground_material = Material::Lambertian {
        albedo: Vector3::new(0.5, 0.5, 0.5),
    };
    let mut world: Vec<Surface> = vec![Surface::new(
        Geometry::sphere(Vector3::new(0.0, -1000.0, 0.0), 1000.0)?,
        ground_material,
    )];

    let big_spheres = {
        let back_sphere = Surface::new(
            Geometry::sphere(Vector3::new(-4.0, 1.0, 0.0), BIG_SPHERES_RADIUS)?,
            Material::Lambertian {
                albedo: Vector3::new(0.4, 0.2, 0.1),
            },
        );

        let middle_sphere = Surface::new(
            Geometry::sphere(Vector3::new(0.0, 1.0, 0.0), BIG_SPHERES_RADIUS)?,
            Material::Dielectric {
                refraction_index: 1.5,
            },
        );

        let front_sphere = Surface::new(
            Geometry::sphere(Vector3::new(4.0, 1.0, 0.0), BIG_SPHERES_RADIUS)?,
            Material::Metal {
                albedo: Vector3::new(0.7, 0.6, 0.5),
                fuzz_radius: 0.0,
            },
        );

        vec![back_sphere, middle_sphere, front_sphere]
    };

    for a in -11..11 {
        for b in -11..11 {
            let center = Vector3::new(
                a as f64 + 0.9 * random::<f64>(),
                SMALL_SPHERES_RADIUS,
                b as f64 + 0.9 * random::<f64>(),
            );

            if big_spheres
                .iter()
                .map(|surface| match surface.geometry {
                    Geometry::Sphere {
                        center: sphere_center,
                        ..
                    } => (sphere_center - center).length(),
                    Geometry::Quadrilateral {
                        q: _,
                        u: _,
                        v: _,
                        norm: _,
                        d: _,
                        w: _,
                    } => unreachable!(),
                })
                .any(|dist_between_centers| {
                    dist_between_centers < (BIG_SPHERES_RADIUS + SMALL_SPHERES_RADIUS)
                })
            {
                continue;
            }

            let material = {
                let choose_material = random::<f64>();

                if choose_material < 0.8 {
                    Material::Lambertian {
                        albedo: Vector3::random() * Vector3::random(),
                    }
                } else if choose_material < 0.95 {
                    Material::Metal {
                        albedo: Vector3::random_range(Interval::new(0.5, 1.0)),
                        fuzz_radius: random_range(0.0..0.5),
                    }
                } else {
                    Material::Dielectric {
                        refraction_index: 1.5,
                    }
                }
            };

            world.push(Surface::new(
                Geometry::sphere(center, SMALL_SPHERES_RADIUS)?,
                material,
            ));
        }
    }

    big_spheres
        .into_iter()
        .for_each(|surface| world.push(surface));

    Ok(world.into_boxed_slice())
}
