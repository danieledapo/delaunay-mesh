use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Bbox {
    min: Vec2,
    max: Vec2,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BarycentricCoords {
    w0: f64,
    w1: f64,
    w2: f64,
}

impl Vec2 {
    pub fn zero() -> Self {
        Vec2::new(0.0, 0.0)
    }

    pub fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    pub fn dist(&self, p: Vec2) -> f64 {
        self.dist2(p).sqrt()
    }

    pub fn dist2(&self, p: Vec2) -> f64 {
        (*self - p).norm2()
    }

    pub fn norm(&self) -> f64 {
        self.norm2().sqrt()
    }

    pub fn norm2(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2)
    }
}

impl Bbox {
    pub fn new(p: Vec2) -> Self {
        Bbox { min: p, max: p }
    }

    pub fn min(&self) -> Vec2 {
        self.min
    }

    pub fn max(&self) -> Vec2 {
        self.max
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    pub fn split(&self, p: Vec2) -> [Bbox; 4] {
        debug_assert!(self.contains(p));

        [
            Bbox {
                min: self.min,
                max: p,
            },
            Bbox {
                min: Vec2::new(p.x, self.min.y),
                max: Vec2::new(self.max.x, p.y),
            },
            Bbox {
                min: Vec2::new(self.min.x, p.y),
                max: Vec2::new(p.x, self.max.y),
            },
            Bbox {
                min: p,
                max: self.max,
            },
        ]
    }

    pub fn expand(&mut self, p: Vec2) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);

        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    pub fn enlarge(&mut self, amount: f64) {
        self.min.x -= amount;
        self.min.y -= amount;

        self.max.x += amount;
        self.max.y += amount;
    }

    pub fn contains(&self, p: Vec2) -> bool {
        self.min.x <= p.x && self.min.y <= p.y && self.max.x >= p.x && self.max.y >= p.y
    }

    pub fn intersection(&self, other: Bbox) -> Option<Bbox> {
        let min_x = self.min.x.max(other.min.x);
        let min_y = self.min.y.max(other.min.y);
        let max_x = self.max.x.min(other.max.x);
        let max_y = self.max.y.min(other.max.y);

        if min_x > max_x || min_y > max_y {
            None
        } else {
            Some(Bbox {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            })
        }
    }

    pub fn dimensions(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn area(&self) -> f64 {
        let d = self.dimensions();
        d.x * d.y
    }
}

impl Circle {
    pub fn new(center: Vec2, radius: f64) -> Self {
        Circle { center, radius }
    }

    pub fn circumcircle(a: Vec2, b: Vec2, c: Vec2) -> Self {
        // https://en.wikipedia.org/wiki/Circumscribed_circle#Cartesian_coordinates_2
        let b = b - a;
        let c = c - a;

        let d = 2.0 * (b.x * c.y - b.y * c.x);
        let x = (c.y * (b.x.powi(2) + b.y.powi(2)) - b.y * (c.x.powi(2) + c.y.powi(2))) / d;
        let y = (b.x * (c.x.powi(2) + c.y.powi(2)) - c.x * (b.x.powi(2) + b.y.powi(2))) / d;

        Circle::new(a + Vec2::new(x, y), Vec2::new(x, y).norm())
    }

    pub fn contains(&self, p: Vec2) -> bool {
        self.center.dist(p) - self.radius <= 1e-6
    }

    pub fn bbox(&self) -> Bbox {
        let mut b = Bbox::new(self.center);
        b.enlarge(self.radius);
        b
    }
}

impl BarycentricCoords {
    pub fn triangle([a, b, c]: [Vec2; 3], p: Vec2) -> Option<Self> {
        let d = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);

        let w0 = ((b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y)) / d;
        let w1 = ((c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y)) / d;
        let w2 = 1.0 - w0 - w1;

        if w0 + w1 + w2 > 1.0 {
            None
        } else {
            Some(BarycentricCoords { w0, w1, w2 })
        }
    }

    pub fn to_point(&self, triangle: [Vec2; 3]) -> Vec2 {
        triangle[0] * self.w0 + triangle[1] * self.w1 + triangle[2] * self.w2
    }

    pub fn interpolate(&self, vals: [f64; 3]) -> f64 {
        vals[0] * self.w0 + vals[1] * self.w1 + vals[2] * self.w2
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(mut self, rhs: Vec2) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

impl Add<f64> for Vec2 {
    type Output = Vec2;

    fn add(mut self, rhs: f64) -> Self::Output {
        self.x += rhs;
        self.y += rhs;
        self
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(mut self, rhs: Vec2) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self
    }
}

impl Sub<f64> for Vec2 {
    type Output = Vec2;

    fn sub(mut self, rhs: f64) -> Self::Output {
        self.x -= rhs;
        self.y -= rhs;
        self
    }
}

impl Mul for Vec2 {
    type Output = Vec2;

    fn mul(mut self, rhs: Vec2) -> Self::Output {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self.x *= rhs;
        self.y *= rhs;
        self
    }
}

impl Div for Vec2 {
    type Output = Vec2;

    fn div(mut self, rhs: Vec2) -> Self::Output {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self
    }
}

impl Div<f64> for Vec2 {
    type Output = Vec2;

    fn div(mut self, rhs: f64) -> Self::Output {
        self.x /= rhs;
        self.y /= rhs;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circumcircle() {
        assert_eq!(
            Circle::circumcircle(Vec2::zero(), Vec2::new(3.0, 4.0), Vec2::new(0.0, 4.0)),
            Circle::new(Vec2::new(1.5, 2.0), 2.5)
        );

        assert_eq!(
            Circle::circumcircle(
                Vec2::new(1.0, 1.0),
                Vec2::new(4.0, 5.0),
                Vec2::new(1.0, 5.0)
            ),
            Circle::new(Vec2::new(2.5, 3.0), 2.5)
        );
    }

}
