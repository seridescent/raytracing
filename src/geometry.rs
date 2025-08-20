use thiserror::Error;

use crate::{
    aabb::AABB,
    interval::Interval,
    ray::Ray,
    vector::{Vector3, cross, dot},
};

#[derive(Clone)]
pub struct Hit {
    pub t: f64,
    pub p: Vector3,

    /// u-coordinate
    pub alpha: f64,
    /// v-coordinate
    pub beta: f64,

    /// whether the ray hit the "outward" face of this surface
    pub front_face: bool,
    pub face_normal: Vector3,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Geometry {
    Sphere {
        center: Vector3,
        radius: f64,
    },
    Quadrilateral {
        q: Vector3,
        u: Vector3,
        v: Vector3,
        norm: Vector3,
        d: f64,
        w: Vector3,
    },
    Triangle {
        q: Vector3,
        u: Vector3,
        v: Vector3,
        norm: Vector3,
        d: f64,
        w: Vector3,
    },
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

    pub fn quadrilateral(q: Vector3, u: Vector3, v: Vector3) -> Self {
        let n = cross(u, v);
        let norm = n.to_unit();
        Self::Quadrilateral {
            q,
            u,
            v,
            norm,
            d: dot(norm, q),
            w: n / dot(n, n),
        }
    }

    pub fn triangle(q: Vector3, u: Vector3, v: Vector3) -> Self {
        let n = cross(u, v);
        let norm = n.to_unit();
        Self::Triangle {
            q,
            u,
            v,
            norm,
            d: dot(norm, q),
            w: n / dot(n, n),
        }
    }

    pub fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        match *self {
            Geometry::Sphere { center, radius } => sphere::hit(center, radius, ray, ray_t),
            Geometry::Quadrilateral {
                q,
                u,
                v,
                norm,
                d,
                w,
            } => quad::hit(q, u, v, norm, d, w, ray, ray_t),
            Geometry::Triangle {
                q,
                u,
                v,
                norm,
                d,
                w,
            } => triangle::hit(q, u, v, norm, d, w, ray, ray_t),
        }
    }

    pub fn bounding_box(&self) -> AABB {
        match *self {
            Geometry::Sphere { center, radius } => sphere::bounding_box(center, radius),
            Geometry::Quadrilateral {
                q,
                u,
                v,
                norm: _,
                d: _,
                w: _,
            } => quad::bounding_box(q, u, v),
            Geometry::Triangle {
                q,
                u,
                v,
                norm: _,
                d: _,
                w: _,
            } => triangle::bounding_box(q, u, v),
        }
    }
}

fn compute_face_normal(ray: &Ray, outward_normal: Vector3) -> (bool, Vector3) {
    let front_face = dot(ray.direction, outward_normal) < 0.0;

    let face_normal = if front_face {
        outward_normal
    } else {
        -outward_normal
    };

    (front_face, face_normal)
}

fn plane_intersection(norm: Vector3, d: f64, ray: &Ray) -> Option<f64> {
    let denominator = dot(norm, ray.direction);
    if denominator.abs() < 1e-10 {
        return None;
    }

    Some((d - dot(norm, ray.origin)) / denominator)
}

struct UvHit {
    t: f64,
    p: Vector3,

    /// u-coordinate
    alpha: f64,
    /// v-coordinate
    beta: f64,
}

fn uv_hit(
    q: Vector3,
    u: Vector3,
    v: Vector3,
    norm: Vector3,
    d: f64,
    w: Vector3,
    ray: &Ray,
    ray_t: &Interval,
) -> Option<UvHit> {
    let t = plane_intersection(norm, d, ray)?;

    if !ray_t.contains(t) {
        return None;
    }

    let p = ray.at(t);

    // uv coordinates
    let qp = p - q;
    let alpha = dot(w, cross(qp, v));
    let beta = dot(w, cross(u, qp));

    Some(UvHit { t, p, alpha, beta })
}

mod sphere {
    use std::f64::consts::PI;

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

        let theta = f64::acos(-p.y);
        let phi = f64::atan2(-p.z, p.x) + PI;

        let alpha = phi / (2.0 * PI);
        let beta = theta / PI;

        Some(Hit {
            t,
            p,
            alpha,
            beta,
            face_normal,
            front_face,
        })
    }

    pub fn bounding_box(center: Vector3, radius: f64) -> AABB {
        let radii = Vector3::new(radius, radius, radius);
        AABB::new(center + radii, center - radii)
    }
}

mod quad {
    use crate::{aabb::AABB, interval::Interval, ray::Ray, vector::Vector3};

    use super::{Hit, UvHit, compute_face_normal, uv_hit};

    #[allow(clippy::too_many_arguments)]
    pub fn hit(
        q: Vector3,
        u: Vector3,
        v: Vector3,
        norm: Vector3,
        d: f64,
        w: Vector3,
        ray: &Ray,
        ray_t: &Interval,
    ) -> Option<Hit> {
        let UvHit { t, p, alpha, beta } = uv_hit(q, u, v, norm, d, w, ray, ray_t)?;

        if !Interval::UNIT.contains(alpha) || !Interval::UNIT.contains(beta) {
            return None;
        }

        let (front_face, face_normal) = compute_face_normal(ray, norm);
        Some(Hit {
            t,
            p,
            alpha,
            beta,
            face_normal,
            front_face,
        })
    }

    pub fn bounding_box(q: Vector3, u: Vector3, v: Vector3) -> AABB {
        AABB::new(q, q + u + v).padded(0.0001)
    }
}

mod triangle {
    use crate::{aabb::AABB, interval::Interval, ray::Ray, vector::Vector3};

    use super::{Hit, UvHit, compute_face_normal, uv_hit};

    #[allow(clippy::too_many_arguments)]
    pub fn hit(
        q: Vector3,
        u: Vector3,
        v: Vector3,
        norm: Vector3,
        d: f64,
        w: Vector3,
        ray: &Ray,
        ray_t: &Interval,
    ) -> Option<Hit> {
        let UvHit { t, p, alpha, beta } = uv_hit(q, u, v, norm, d, w, ray, ray_t)?;

        if !(alpha >= 0.0 && beta >= 0.0 && alpha + beta <= 1.0) {
            return None;
        }

        let (front_face, face_normal) = compute_face_normal(ray, norm);
        Some(Hit {
            t,
            p,
            alpha,
            beta,
            face_normal,
            front_face,
        })
    }

    pub fn bounding_box(q: Vector3, u: Vector3, v: Vector3) -> AABB {
        AABB::merge(AABB::new(q, q + u), AABB::new(q, q + v)).padded(0.0001)
    }
}
