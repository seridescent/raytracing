use thiserror::Error;

use crate::{
    aabb::AABB,
    interval::Interval,
    ray::Ray,
    vector::{Vector3, dot},
};

#[derive(Clone)]
pub struct Hit {
    pub t: f64,
    pub p: Vector3,
    pub face_normal: Vector3,

    /// whether the ray hit the "outward" face of this surface
    pub front_face: bool,
}

pub enum Geometry {
    Sphere { center: Vector3, radius: f64 },
}

#[derive(Error, Debug)]
pub enum ConstructSphereError {
    #[error("invalid radius {0} (expected non-negative radius)")]
    NonnegativeRadius(f64),
}

impl Geometry {
    pub fn sphere(center: Vector3, radius: f64) -> Result<Self, ConstructSphereError> {
        if radius < 0.0 {
            Err(ConstructSphereError::NonnegativeRadius(radius))
        } else {
            Ok(Geometry::Sphere { center, radius })
        }
    }

    pub fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        match *self {
            Geometry::Sphere { center, radius } => sphere::hit(center, radius, ray, ray_t),
        }
    }

    pub fn bounding_box(&self) -> AABB {
        match *self {
            Geometry::Sphere { center, radius } => sphere::bounding_box(center, radius),
        }
    }
}

pub fn compute_face_normal(ray: &Ray, outward_normal: Vector3) -> (bool, Vector3) {
    let front_face = dot(ray.direction, outward_normal) < 0.0;

    let face_normal = if front_face {
        outward_normal
    } else {
        -outward_normal
    };

    (front_face, face_normal)
}

mod sphere {
    use crate::{
        aabb::AABB,
        interval::Interval,
        ray::Ray,
        vector::{Vector3, dot},
    };

    use super::{Hit, compute_face_normal};

    pub fn hit(center: Vector3, radius: f64, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        let oc = center - ray.origin;
        let a = ray.direction.length_squared();
        let h = dot(ray.direction, oc);
        let c = oc.length_squared() - radius.powi(2);

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
        let outward_normal = (p - center) / radius;
        let (front_face, face_normal) = compute_face_normal(ray, outward_normal);

        Some(Hit {
            t,
            p,
            face_normal,
            front_face,
        })
    }

    pub fn bounding_box(center: Vector3, radius: f64) -> AABB {
        let radii = Vector3::new(radius, radius, radius);
        AABB::new(center + radii, center - radii)
    }
}
