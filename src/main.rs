use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use rand::{random, random_range};
use raytracing::camera::Camera;
use raytracing::hittable::Hittable;
use raytracing::interval::Interval;
use raytracing::material::{Dielectric, Lambertian, Material, Metal};
use raytracing::sphere::{ConstructSphereError, Sphere};
use raytracing::vector::Vector3;

#[allow(dead_code)]
fn demo_spheres() -> Result<Vec<Box<dyn Hittable>>, ConstructSphereError> {
    let material_ground = Arc::new(Lambertian::new(Vector3::new(0.8, 0.8, 0.0)));
    let material_center = Arc::new(Lambertian::new(Vector3::new(0.1, 0.2, 0.5)));
    let material_left = Arc::new(Dielectric::new(1.5));
    let material_bubble = Arc::new(Dielectric::new(1.0 / 1.5));
    let material_right = Arc::new(Metal::new(Vector3::new(0.8, 0.6, 0.2), 1.0));

    Ok(vec![
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
    ])
}

#[allow(dead_code)]
fn cover_spheres() -> Result<Vec<Box<dyn Hittable>>, ConstructSphereError> {
    const SMALL_SPHERES_RADIUS: f64 = 0.2;
    const BIG_SPHERES_RADIUS: f64 = 1.0;

    let ground_material = Arc::new(Lambertian::new(Vector3::new(0.5, 0.5, 0.5)));
    let mut world: Vec<Box<dyn Hittable>> = vec![Box::new(Sphere::new(
        Vector3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )?)];

    let big_spheres = {
        let back_sphere = Sphere::new(
            Vector3::new(-4.0, 1.0, 0.0),
            BIG_SPHERES_RADIUS,
            Arc::new(Lambertian::new(Vector3::new(0.4, 0.2, 0.1))),
        )?;

        let middle_sphere = Sphere::new(
            Vector3::new(0.0, 1.0, 0.0),
            BIG_SPHERES_RADIUS,
            Arc::new(Dielectric::new(1.5)),
        )?;

        let front_sphere = Sphere::new(
            Vector3::new(4.0, 1.0, 0.0),
            BIG_SPHERES_RADIUS,
            Arc::new(Metal::new(Vector3::new(0.7, 0.6, 0.5), 0.0)),
        )?;

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
                .map(|sphere| (sphere.center - center).length())
                .any(|dist_between_centers| {
                    dist_between_centers < (BIG_SPHERES_RADIUS + SMALL_SPHERES_RADIUS)
                })
            {
                // skip small spheres that would overlap with any of the big spheres
                continue;
            }

            let material: Arc<dyn Material> = {
                let choose_material = random::<f64>();

                if choose_material < 0.8 {
                    Arc::new(Lambertian::new(Vector3::random() * Vector3::random()))
                } else if choose_material < 0.95 {
                    Arc::new(Metal::new(
                        Vector3::random_range(Interval::new(0.5, 1.0)),
                        random_range(0.0..0.5),
                    ))
                } else {
                    Arc::new(Dielectric::new(1.5))
                }
            };

            world.push(Box::new(Sphere::new(
                center,
                SMALL_SPHERES_RADIUS,
                material,
            )?));
        }
    }

    big_spheres
        .into_iter()
        .map(Box::new)
        .for_each(|boxed| world.push(boxed));

    Ok(world)
}

fn main() -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();

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

        ..Default::default()
    };

    camera.initialize().render(&world);

    eprintln!(
        "\n\nDone! Ran for {:#?}",
        Instant::now().duration_since(start_time)
    );

    Ok(())
}
