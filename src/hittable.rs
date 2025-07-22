use std::rc::Rc;

use crate::{
    interval::Interval,
    material::Material,
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

    pub material: Rc<dyn Material>,
}

pub trait Hittable {
    fn hit(&self, ray: Ray, ray_t: Interval) -> Option<Hit>;
}

impl Hittable for Vec<Box<dyn Hittable>> {
    fn hit(&self, ray: Ray, ray_t: Interval) -> Option<Hit> {
        self.iter().fold(None, |acc, e| {
            let maybe_hit = e.hit(ray, ray_t);

            match (acc, maybe_hit) {
                (None, None) => None,
                (None, Some(hit)) => Some(hit),
                (Some(best_hit), None) => Some(best_hit),
                (Some(best_hit), Some(hit)) => {
                    Some(if hit.t < best_hit.t { hit } else { best_hit })
                }
            }
        })
    }
}

pub fn compute_face_normal(ray: Ray, outward_normal: Vector3) -> (bool, Vector3) {
    let front_face = dot(ray.direction, outward_normal) < 0.0;

    let face_normal = if front_face {
        outward_normal
    } else {
        -outward_normal
    };

    (front_face, face_normal)
}
