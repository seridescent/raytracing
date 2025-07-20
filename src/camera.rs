use std::f64::INFINITY;

use crate::{hittable::Hittable, interval::Interval, ray::Ray, vector::Vector3};

#[derive(Default)]
pub struct Camera {
    aspect_ratio: f64,
    image_width: u32,

    // Derived
    image_height: u32,
    center: Vector3,
    pixel00_loc: Vector3,
    pixel_du: Vector3,
    pixel_dv: Vector3,
}

pub struct InitializedCamera(Camera);

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: u32) -> Self {
        Self {
            aspect_ratio,
            image_width,
            ..Default::default()
        }
    }

    pub fn initialize(&self) -> InitializedCamera {
        let aspect_ratio = self.aspect_ratio;
        let image_width = self.image_width;
        let image_height = {
            let h = (self.image_width as f64 / self.aspect_ratio) as u32;
            if h < 1 { 1 } else { h }
        };

        let viewport_height = 2.0;
        let viewport_width = viewport_height * image_width as f64 / image_height as f64;

        let focal_length = 1.0;
        let center = Vector3::ZERO;

        let viewport_u = Vector3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vector3::new(0.0, -viewport_height, 0.0);

        let pixel_du = viewport_u / image_width as f64;
        let pixel_dv = viewport_v / image_height as f64;

        let viewport_upper_left =
            center - Vector3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_du + pixel_dv) * 0.5;

        InitializedCamera(Self {
            aspect_ratio,
            image_width,
            image_height,
            center,
            pixel00_loc,
            pixel_du,
            pixel_dv,
        })
    }
}

impl InitializedCamera {
    pub fn render(&self, world: &impl Hittable) {
        let camera = &self.0;

        println!("P3");
        println!("{} {}", camera.image_width, camera.image_height);
        println!("{}", 255);

        for row in 0..camera.image_height {
            eprint!("\rScanlines remaining: {}", camera.image_height - row);
            for col in 0..camera.image_width {
                let pixel_center = camera.pixel00_loc
                    + (col as f64 * camera.pixel_du)
                    + (row as f64 * camera.pixel_dv);
                let ray = Ray::new(camera.center, pixel_center - camera.center);

                let color = ray_color(ray, world);

                println!("{}", ppm_pixel(color))
            }
        }
    }
}

fn ray_color(ray: Ray, world: &impl Hittable) -> Vector3 {
    if let Some(hit) = world.hit(ray, Interval::new(0.0, INFINITY)) {
        return (hit.face_normal + Vector3::new(1.0, 1.0, 1.0)) * 0.5;
    }

    let alpha = (ray.direction.to_unit().y + 1.0) * 0.5;

    let white = Vector3::new(1.0, 1.0, 1.0);
    let blue = Vector3::new(0.5, 0.7, 1.0);

    (1.0 - alpha) * white + alpha * blue
}

fn ppm_pixel(color: Vector3) -> String {
    let ir = (255.999 * color.x) as u8;
    let ig = (255.999 * color.y) as u8;
    let ib = (255.999 * color.z) as u8;

    format!("{ir} {ig} {ib}")
}
