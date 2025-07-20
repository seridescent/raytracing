use std::{cell::RefCell, rc::Rc};

use rand::{Rng, rngs::StdRng};

use crate::{hittable::Hittable, interval::Interval, ray::Ray, vector::Vector3};

pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
}

pub struct InitializedCamera {
    image_width: u32,
    samples_per_pixel: u32,
    max_depth: u32,

    // Derived
    image_height: u32,
    pixel_samples_scale: f64,
    center: Vector3,
    pixel00_loc: Vector3,
    pixel_du: Vector3,
    pixel_dv: Vector3,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
        }
    }
}

impl Camera {
    pub fn initialize(self) -> InitializedCamera {
        let image_height = {
            let h = (self.image_width as f64 / self.aspect_ratio) as u32;
            if h < 1 { 1 } else { h }
        };

        let pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        let viewport_height = 2.0;
        let viewport_width = viewport_height * self.image_width as f64 / image_height as f64;

        let focal_length = 1.0;
        let center = Vector3::ZERO;

        let viewport_u = Vector3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vector3::new(0.0, -viewport_height, 0.0);

        let pixel_du = viewport_u / self.image_width as f64;
        let pixel_dv = viewport_v / image_height as f64;

        let viewport_upper_left =
            center - Vector3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_du + pixel_dv) * 0.5;

        InitializedCamera {
            image_width: self.image_width,
            samples_per_pixel: self.samples_per_pixel,
            max_depth: self.max_depth,
            image_height,
            center,
            pixel00_loc,
            pixel_du,
            pixel_dv,
            pixel_samples_scale,
        }
    }
}

impl InitializedCamera {
    pub fn render(&self, rng: Rc<RefCell<StdRng>>, world: &impl Hittable) {
        println!("P3");
        println!("{} {}", self.image_width, self.image_height);
        println!("{}", 255);

        for row in 0..self.image_height {
            eprint!("\rScanlines remaining: {}   ", self.image_height - row);
            for col in 0..self.image_width {
                let color = (0..self.samples_per_pixel)
                    .map(|_| sample_square(&mut *rng.borrow_mut()))
                    .map(|offset| self.get_ray(col, row, offset))
                    .map(|ray| ray_color(&mut *rng.borrow_mut(), ray, world, self.max_depth))
                    .fold(Vector3::ZERO, |acc, e| acc + e);

                println!("{}", ppm_pixel(color * self.pixel_samples_scale))
            }
        }
    }

    fn get_ray(&self, col: u32, row: u32, offset: Vector3) -> Ray {
        let pixel_sample = self.pixel00_loc
            + ((col as f64 + offset.x) * self.pixel_du)
            + ((row as f64 + offset.y) * self.pixel_dv);

        Ray::new(self.center, pixel_sample - self.center)
    }
}

fn sample_square(rng: &mut impl Rng) -> Vector3 {
    Vector3::new(rng.random::<f64>() - 0.5, rng.random::<f64>() - 0.5, 0.0)
}

fn ray_color(
    rng: &mut impl Rng,
    ray: Ray,
    world: &impl Hittable,
    remaining_ray_bounces: u32,
) -> Vector3 {
    if remaining_ray_bounces <= 0 {
        return Vector3::ZERO;
    }

    if let Some(hit) = world.hit(ray, Interval::new(0.001, f64::INFINITY)) {
        let direction = hit.face_normal + Vector3::random_unit(rng);
        return (ray_color(
            rng,
            Ray::new(hit.p, direction),
            world,
            remaining_ray_bounces - 1,
        )) * 0.5;
    }

    let alpha = (ray.direction.to_unit().y + 1.0) * 0.5;

    let white = Vector3::new(1.0, 1.0, 1.0);
    let blue = Vector3::new(0.5, 0.7, 1.0);

    (1.0 - alpha) * white + alpha * blue
}

fn ppm_pixel(color: Vector3) -> String {
    let intensity = Interval::new(0.0, 0.999);
    let ir = (255.999 * intensity.clamp(color.x)) as u8;
    let ig = (255.999 * intensity.clamp(color.y)) as u8;
    let ib = (255.999 * intensity.clamp(color.z)) as u8;

    format!("{ir} {ig} {ib}")
}
