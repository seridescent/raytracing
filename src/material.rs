use crate::{
    hittable::Hit,
    ray::Ray,
    vector::{Vector3, dot, reflect, refract},
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

#[derive(Clone, Copy)]
pub struct Dielectric {
    /// aka eta_out, "out" for the material that the refracted ray
    /// enters as it "leaves" the refracting boundary
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: Ray, hit: Hit) -> Option<Scatter> {
        let r_in = ray.direction.to_unit();
        let eta_in_over_eta_out = if hit.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let cos_theta = dot(r_in, hit.face_normal).clamp(-1.0, 1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();
        let r_out = if eta_in_over_eta_out * sin_theta > 1.0 {
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
