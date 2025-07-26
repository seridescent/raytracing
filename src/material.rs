use rand::random;

use crate::{
    hittable::Hit,
    ray::Ray,
    vector::{Vector3, dot, reflect, refract},
};

#[derive(Clone, Debug)]
pub struct Scatter {
    pub ray: Ray,
    pub attenuation: Vector3,
}

#[derive(Clone)]
pub enum Material {
    Lambertian { albedo: Vector3 },
    Metal { albedo: Vector3, fuzz_radius: f64 },
    Dielectric { refraction_index: f64 },
}

impl Material {
    pub fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<Scatter> {
        match *self {
            Material::Lambertian { albedo } => {
                let direction = hit.face_normal + Vector3::random_unit();
                let direction = if direction.is_near_zero() {
                    hit.face_normal
                } else {
                    direction
                };

                Some(Scatter {
                    ray: Ray::new(hit.p, direction),
                    attenuation: albedo,
                })
            }
            Material::Metal {
                albedo,
                fuzz_radius,
            } => {
                let reflected = reflect(ray.direction, hit.face_normal);
                let fuzz = Vector3::random_unit() * fuzz_radius;
                let fuzzed = reflected.to_unit() + fuzz;
                if dot(fuzzed, hit.face_normal) > 0.0 {
                    Some(Scatter {
                        ray: Ray::new(hit.p, fuzzed),
                        attenuation: albedo,
                    })
                } else {
                    None
                }
            }
            Material::Dielectric { refraction_index } => {
                let r_in = ray.direction.to_unit();
                let eta_in_over_eta_out = if hit.front_face {
                    1.0 / refraction_index
                } else {
                    refraction_index
                };

                let cos_theta = dot(-r_in, hit.face_normal).clamp(-1.0, 1.0);
                let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();
                let r_out = if eta_in_over_eta_out * sin_theta > 1.0
                    || reflectance(cos_theta, eta_in_over_eta_out) > random::<f64>()
                {
                    reflect(r_in, hit.face_normal)
                } else {
                    refract(r_in, hit.face_normal, eta_in_over_eta_out)
                };

                Some(Scatter {
                    ray: Ray::new(hit.p, r_out),
                    attenuation: Vector3::new(1.0, 1.0, 1.0),
                })
            }
        }
    }
}

/// Schlick's approximation for reflectance
fn reflectance(cos_theta: f64, refraction_index: f64) -> f64 {
    let r0 = ((1.0 - refraction_index) / (1.0 + refraction_index)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
}
