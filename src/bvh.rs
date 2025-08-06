use crate::{
    aabb::AABB,
    geometry::Hit,
    interval::Interval,
    material::Material,
    ray::Ray,
    surface::{Hittable, Surface},
};

pub struct BVH {}

impl BVH {
    pub fn from_slice(surfaces: &[Surface]) -> Self {
        todo!()
    }
}

impl Hittable for BVH {
    fn hit(&self, _ray: &Ray, _ray_t: &Interval) -> Option<(Hit, Material)> {
        todo!()
    }

    fn bounding_box(&self) -> AABB {
        todo!()
    }
}
