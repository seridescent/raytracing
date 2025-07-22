use crate::{
    hittable::Hit,
    ray::Ray,
    vector::{Vector3, reflect},
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
}

impl Metal {
    pub fn new(albedo: Vector3) -> Self {
        Self { albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: Ray, hit: Hit) -> Option<Scatter> {
        Some(Scatter {
            ray: Ray::new(hit.p, reflect(ray.direction, hit.face_normal)),
            attenuation: self.albedo,
        })
    }
}
