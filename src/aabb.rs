use std::f64;

use crate::{interval::Interval, ray::Ray, vector::Vector3};

pub struct AABB {
    min: Vector3,
    max: Vector3,
}

impl AABB {
    pub const EMPTY: Self = Self {
        min: Vector3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
        max: Vector3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
    };

    pub fn new(a: Vector3, b: Vector3) -> Self {
        Self {
            min: Vector3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: Vector3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    pub fn merge(a: AABB, b: AABB) -> Self {
        Self {
            min: Vector3::new(
                a.min.x.min(b.min.x),
                a.min.y.min(b.min.y),
                a.min.z.min(b.min.z),
            ),
            max: Vector3::new(
                a.max.x.max(b.max.x),
                a.max.y.max(b.max.y),
                a.max.z.max(b.max.z),
            ),
        }
    }

    pub fn hit(&self, ray: &Ray, ray_t: &Interval) -> bool {
        let t_0 = (self.min - ray.origin) / ray.direction;
        let t_1 = (self.max - ray.origin) / ray.direction;

        let lowers = [t_0.x.min(t_1.x), t_0.y.min(t_1.y), t_0.z.min(t_1.z)];
        let uppers = [t_0.x.max(t_1.x), t_0.y.max(t_1.y), t_0.z.max(t_1.z)];

        if lowers.contains(&f64::NAN) || uppers.contains(&f64::NAN) {
            return false;
        }

        let lowers_max = lowers
            .iter()
            .map(|&t_lower| ray_t.clamp(t_lower))
            .fold(f64::NAN, |acc, e| acc.max(e));
        let uppers_min = uppers
            .iter()
            .map(|&t_lower| ray_t.clamp(t_lower))
            .fold(f64::NAN, |acc, e| acc.min(e));

        lowers_max < uppers_min
    }
}
