use thiserror::Error;

use crate::{
    hittable::{Hit, Hittable, compute_face_normal},
    interval::Interval,
    ray::Ray,
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
    fn hit(&self, ray: Ray, ray_t: Interval) -> Option<Hit> {
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

            let root = (h - sqrtd) / a;

            if ray_t.surrounds(root) {
                root
            } else {
                let root = (h + sqrtd) / a;
                if !ray_t.surrounds(root) {
                    return None;
                }

                root
            }
        };

        let p = ray.at(t);
        let outward_normal = (p - self.center) / self.radius;
        let (front_face, face_normal) = compute_face_normal(ray, outward_normal);

        Some(Hit {
            t,
            p,
            face_normal,
            front_face,
        })
    }
}
