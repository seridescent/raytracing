use crate::{
    ray::Ray,
    vector::{Vector3, dot},
};

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub t: f64,
    pub p: Vector3,
    pub face_normal: Vector3,

    /// whether the ray hit the "outward" face of this surface
    pub front_face: bool,
}

pub trait Hittable {
    fn hit(self, ray: Ray, t_min: f64, t_max: f64) -> Option<Hit>;
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
