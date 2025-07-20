use crate::{ray::Ray, vector::Vector3};

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub t: f64,
    pub p: Vector3,
    pub normal: Vector3,
}

pub trait Hittable {
    fn hit(self, ray: Ray, t_min: f64, t_max: f64) -> Option<Hit>;
}
