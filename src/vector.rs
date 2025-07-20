use std::ops;

use rand::Rng;

use crate::interval::Interval;

#[derive(Copy, Clone, Debug)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for Vector3 {
    fn default() -> Self {
        Vector3::ZERO
    }
}

impl Vector3 {
    pub const ZERO: Self = Self {
        x: 0.,
        y: 0.,
        z: 0.,
    };

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn to_unit(self) -> Self {
        self / self.length()
    }

    pub fn random_range(rng: &mut impl Rng, range: Interval) -> Self {
        Self {
            x: rng.random_range(range.min..range.max),
            y: rng.random_range(range.min..range.max),
            z: rng.random_range(range.min..range.max),
        }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            x: rng.random::<f64>(),
            y: rng.random::<f64>(),
            z: rng.random::<f64>(),
        }
    }

    pub fn random_unit(rng: &mut impl Rng) -> Self {
        loop {
            let candidate = Self::random_range(rng, Interval::new(-1.0, 1.0));
            let lensq = candidate.length_squared();

            // there exist candidate vectors s.t. candidate.length_squared() == 0.0
            // because tiny_float ^ 2 can underflow to 0.0.
            // we have to reject such candidates, or else we will produce "unit" vectors [inf inf inf].
            //
            // the book rejects additional candidate vectors with extremely small values of lensq,
            // and the book uses 1e-160 for this value. however, it seems to me that even
            // subnormal positive lensq values (e.g. `1e-320f64`) produce valid unit vectors.
            //
            // of course, some small samples working out on my macbook does not mean that
            // it's a good idea to widen the range to 0 < lensq <= 1.0, as i'm not confident in a wider
            // range's correctness and this codepath's relevance probably pales in comparison to lighting
            // computations anyway. maybe something to explore another time though.
            //
            if 1e-160 < lensq && lensq <= 1.0 {
                return candidate / lensq.sqrt();
            }
        }
    }

    pub fn random_on_hemisphere(rng: &mut impl Rng, normal: Self) -> Self {
        let on_unit_sphere = Self::random_unit(rng);
        if dot(on_unit_sphere, normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }
}

impl ops::Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Mul for Vector3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::Mul<f64> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul<Vector3> for f64 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Self::Output {
        rhs * self
    }
}

impl ops::MulAssign for Vector3 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl ops::MulAssign<f64> for Vector3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl ops::Div<f64> for Vector3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        let inv = 1.0 / rhs;
        Self {
            x: self.x * inv,
            y: self.y * inv,
            z: self.z * inv,
        }
    }
}

impl ops::DivAssign<f64> for Vector3 {
    fn div_assign(&mut self, rhs: f64) {
        *self *= 1.0 / rhs;
    }
}

impl ops::Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::SubAssign for Vector3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl ops::Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

pub fn dot(lhs: Vector3, rhs: Vector3) -> f64 {
    lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z
}
