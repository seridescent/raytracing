use ray::Ray;
use vector::dot;

use crate::vector::Vector3;

pub mod ray;
pub mod vector;

fn hit_sphere(center: Vector3, radius: f64, ray: Ray) -> f64 {
    let oc = center - ray.origin;
    let a = dot(ray.direction, ray.direction);
    let b = -2.0 * dot(ray.direction, oc);
    let c = dot(oc, oc) - radius.powi(2);

    let discriminant = b.powi(2) - 4.0 * a * c;

    if discriminant < 0.0 {
        -1.0
    } else {
        (-b - discriminant.sqrt()) / (2.0 * a)
    }
}

fn ray_color(ray: Ray) -> Vector3 {
    let center = Vector3::new(0.0, 0.0, -1.0);
    let radius = 0.5;
    let t = hit_sphere(center, radius, ray);

    if t > 0.0 {
        let norm = (ray.at(t) - center) / radius;
        return 0.5 * Vector3::new(norm.x + 1.0, norm.y + 1.0, norm.z + 1.0);
    }

    let alpha = (ray.direction.to_unit().y + 1.0) * 0.5;

    let white = Vector3::new(1.0, 1.0, 1.0);
    let blue = Vector3::new(0.5, 0.7, 1.0);

    (1.0 - alpha) * white + alpha * blue
}

fn main() {
    let ideal_aspect_ratio = 16.0 / 9.0;
    let image_width = 400;

    let image_height = {
        let h = (image_width as f64 / ideal_aspect_ratio) as i32;
        if h < 1 { 1 } else { h }
    };

    // Camera

    let viewport_height = 2.0;
    let viewport_width = viewport_height * image_width as f64 / image_height as f64;

    let focal_length = 1.0;
    let camera_center = Vector3::zero();

    let viewport_u = Vector3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vector3::new(0.0, -viewport_height, 0.0);

    let pixel_du = viewport_u / image_width as f64;
    let pixel_dv = viewport_v / image_height as f64;

    let viewport_upper_left =
        camera_center - Vector3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc = viewport_upper_left + (pixel_du + pixel_dv) * 0.5;

    // Render

    println!("P3");
    println!("{image_width} {image_height}");
    println!("{}", 255);

    for row in 0..image_height {
        eprint!("\rScanlines remaining: {}", image_height - row);
        for col in 0..image_width {
            let pixel_center = pixel00_loc + (col as f64 * pixel_du) + (row as f64 * pixel_dv);
            let ray = Ray::new(camera_center, pixel_center - camera_center);

            let color = ray_color(ray);

            println!("{}", ppm_pixel(color))
        }
    }
}

fn ppm_pixel(color: Vector3) -> String {
    let ir = (255.999 * color.x) as u8;
    let ig = (255.999 * color.y) as u8;
    let ib = (255.999 * color.z) as u8;

    format!("{ir} {ig} {ib}")
}
