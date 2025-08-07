use crate::{
    aabb::AABB,
    geometry::{Geometry, Hit},
    interval::Interval,
    material::Material,
    ray::Ray,
};

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<(Hit, Material)>;
    fn bounding_box(&self) -> AABB;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Surface {
    pub geometry: Geometry,
    pub material: Material,
}

impl Surface {
    pub fn new(geometry: Geometry, material: Material) -> Self {
        Self { geometry, material }
    }
}

impl Hittable for Surface {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<(Hit, Material)> {
        if let Some(hit) = self.geometry.hit(ray, ray_t) {
            return Some((hit, self.material.clone()));
        }

        None
    }

    fn bounding_box(&self) -> AABB {
        self.geometry.bounding_box()
    }
}

impl Hittable for &[Surface] {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<(Hit, Material)> {
        self.iter().fold(None, |acc, e| {
            let maybe_hit = e.hit(ray, ray_t);

            match (acc, maybe_hit) {
                (None, None) => None,
                (None, Some(first)) => Some(first),
                (Some(best), None) => Some(best),
                (Some(best), Some(found)) => Some(if found.0.t < best.0.t { found } else { best }),
            }
        })
    }

    fn bounding_box(&self) -> AABB {
        self.iter()
            .map(|e| e.bounding_box())
            .fold(AABB::EMPTY, AABB::merge)
    }
}
