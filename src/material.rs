use crate::{
    hittable::Hit,
    ray::Ray,
    vector::{Vector3, dot, reflect},
};

#[derive(Clone, Copy, Debug)]
pub struct Scatter {
    pub ray: Ray,
    pub attenuation: Vector3,
}

pub trait Material {
    fn scatter(&self, ray: Ray, hit: Hit) -> Option<Scatter>;
}

#[derive(Clone, Copy)]
pub struct Lambertian {
    albedo: Vector3,
}

impl Lambertian {
    pub fn new(albedo: Vector3) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: Ray, hit: Hit) -> Option<Scatter> {
        let direction = hit.face_normal + Vector3::random_unit();
        let direction = if direction.is_near_zero() {
            hit.face_normal
        } else {
            direction
        };

        Some(Scatter {
            ray: Ray::new(hit.p, direction),
            attenuation: self.albedo,
        })
    }
}

#[derive(Clone, Copy)]
pub struct Metal {
    albedo: Vector3,
    fuzz_radius: f64,
}

impl Metal {
    pub fn new(albedo: Vector3, fuzz_radius: f64) -> Self {
        Self {
            albedo,
            fuzz_radius,
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: Ray, hit: Hit) -> Option<Scatter> {
        let reflected = reflect(ray.direction, hit.face_normal);
        let fuzz = Vector3::random_unit() * self.fuzz_radius;
        let fuzzed = reflected.to_unit() + fuzz;
        if dot(fuzzed, hit.face_normal) > 0.0 {
            Some(Scatter {
                ray: Ray::new(hit.p, fuzzed),
                attenuation: self.albedo,
            })
        } else {
            None
        }
    }
}
