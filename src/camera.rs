use rand::random;
use rayon::prelude::*;

use crate::{
    hittable::Hittable,
    interval::Interval,
    ray::Ray,
    vector::{Vector3, cross},
};

pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,

    pub v_fov: f64,
    pub look_from: Vector3,
    pub look_at: Vector3,
    pub v_up: Vector3,

    pub defocus_angle: f64,
    pub focus_dist: f64,
}

pub struct InitializedCamera {
    image_width: u32,
    samples_per_pixel: u32,
    max_depth: u32,
    defocus_angle: f64,

    image_height: u32,
    pixel_samples_scale: f64,
    center: Vector3,
    pixel00_loc: Vector3,
    pixel_du: Vector3,
    pixel_dv: Vector3,
    defocus_disk_u: Vector3,
    defocus_disk_v: Vector3,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            v_fov: 90.0,
            look_from: Vector3::ZERO,
            look_at: Vector3::new(0.0, 0.0, -1.0),
            v_up: Vector3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
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

        let center = self.look_from;

        let theta = self.v_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;

        let viewport_width = viewport_height * self.image_width as f64 / image_height as f64;

        let w = (self.look_from - self.look_at).to_unit();
        let u = cross(self.v_up, w).to_unit();
        let v = cross(w, u);

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        let pixel_du = viewport_u / self.image_width as f64;
        let pixel_dv = viewport_v / image_height as f64;

        let viewport_upper_left =
            center - (self.focus_dist * w) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_du + pixel_dv) * 0.5;

        let defocus_radius = self.focus_dist * (self.defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        InitializedCamera {
            image_width: self.image_width,
            samples_per_pixel: self.samples_per_pixel,
            max_depth: self.max_depth,
            defocus_angle: self.defocus_angle,
            image_height,
            center,
            pixel00_loc,
            pixel_du,
            pixel_dv,
            pixel_samples_scale,
            defocus_disk_u,
            defocus_disk_v,
        }
    }
}

struct Pixel {
    pub ord: u32,
    pub color: Vector3,
}

impl Pixel {
    pub fn new(ord: u32, color: Vector3) -> Self {
        Self { ord, color }
    }
}

impl InitializedCamera {
    pub fn render(&self, world: &impl Hittable) {
        println!("P3");
        println!("{} {}", self.image_width, self.image_height);
        println!("{}", 255);

        let mut pixels = (0..self.image_height)
            .into_par_iter()
            .flat_map(|row| {
                let world_ref = world;
                (0..self.image_width).into_par_iter().map({
                    move |col| {
                        Pixel::new(
                            row * self.image_width + col,
                            (0..self.samples_per_pixel)
                                .into_par_iter()
                                .map(|_| sample_square())
                                .map(|offset| self.get_ray(col, row, offset))
                                .map(|ray| ray_color(ray, world_ref, self.max_depth))
                                .reduce(|| Vector3::ZERO, |acc, e| acc + e)
                                * self.pixel_samples_scale,
                        )
                    }
                })
            })
            .collect::<Vec<Pixel>>();

        pixels.par_sort_unstable_by_key(|pixel| pixel.ord);

        let body = pixels
            .iter()
            .map(|pixel| ppm_pixel(pixel.color))
            .collect::<Vec<String>>()
            .join("\n");

        println!("{body}");
    }

    fn get_ray(&self, col: u32, row: u32, offset: Vector3) -> Ray {
        let pixel_sample = self.pixel00_loc
            + ((col as f64 + offset.x) * self.pixel_du)
            + ((row as f64 + offset.y) * self.pixel_dv);

        let origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            let p = Vector3::random_in_unit_disk();
            self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
        };

        Ray::new(origin, pixel_sample - origin)
    }
}

fn sample_square() -> Vector3 {
    Vector3::new(random::<f64>() - 0.5, random::<f64>() - 0.5, 0.0)
}

fn ray_color(ray: Ray, world: &impl Hittable, remaining_ray_bounces: u32) -> Vector3 {
    if remaining_ray_bounces == 0 {
        return Vector3::ZERO;
    }

    if let Some(hit) = world.hit(ray, Interval::new(0.001, f64::INFINITY)) {
        return match hit.material.clone().scatter(ray, hit) {
            Some(scatter) => {
                ray_color(scatter.ray, world, remaining_ray_bounces - 1) * scatter.attenuation
            }
            None => Vector3::ZERO,
        };
    }

    let alpha = (ray.direction.to_unit().y + 1.0) * 0.5;

    let white = Vector3::new(1.0, 1.0, 1.0);
    let blue = Vector3::new(0.5, 0.7, 1.0);

    (1.0 - alpha) * white + alpha * blue
}

fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

fn ppm_pixel(color: Vector3) -> String {
    let (r, g, b) = (color.x, color.y, color.z);
    let (r, g, b) = (linear_to_gamma(r), linear_to_gamma(g), linear_to_gamma(b));

    let intensity = Interval::new(0.0, 0.999);
    let ir = (255.999 * intensity.clamp(r)) as u8;
    let ig = (255.999 * intensity.clamp(g)) as u8;
    let ib = (255.999 * intensity.clamp(b)) as u8;

    format!("{ir} {ig} {ib}")
}
