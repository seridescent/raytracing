use thiserror::Error;

use crate::{
    hittable::{Hit, Hittable},
    vector::{Vector3, dot},
};

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    center: Vector3,
    radius: f64,
}

#[derive(Error, Debug)]
pub enum ConstructSphereError {
    #[error("invalid radius {0} (expected non-negative radius)")]
    NonnegativeRadius(f64),
}

impl Sphere {
    pub fn new(center: Vector3, radius: f64) -> Result<Self, ConstructSphereError> {
        if radius < 0.0 {
            Err(ConstructSphereError::NonnegativeRadius(radius))
        } else {
            Ok(Self { center, radius })
        }
    }
}

impl Hittable for Sphere {
    fn hit(self, ray: crate::ray::Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = dot(ray.direction, oc);
        let c = oc.length_squared() - self.radius.powi(2);

        let discriminant = h.powi(2) - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let t = {
            let sqrtd = discriminant.sqrt();

            let is_in_range = |t| !(t < t_min || t >= t_max);

            let root = (h - sqrtd) / a;

            if is_in_range(root) {
                root
            } else {
                let root = (h + sqrtd) / a;
                if !is_in_range(root) {
                    return None;
                }

                root
            }
        };

        let p = ray.at(t);
        let normal = (p - self.center) / self.radius;

        Some(Hit { t, p, normal })
    }
}
