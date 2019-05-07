// -----------------------------------------------------------------------------------------

use std::ops;

// -----------------------------------------------------------------------------------------
// Type
#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// -----------------------------------------------------------------------------------------
// Constants
impl Vec3 {
    pub const UP: Vec3 = Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
}

// -----------------------------------------------------------------------------------------
// Constructor
impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }
}

// -----------------------------------------------------------------------------------------
// Vector arithmetic (via operator traits)
impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl ops::Div<Vec3> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

// -----------------------------------------------------------------------------------------
// Scalar arithmetic (via operator traits)
impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Vec3 {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

// -----------------------------------------------------------------------------------------
// Operations
impl Vec3 {
    pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
        let x = (a.y * b.z) - (a.z * b.y);
        let y = (a.z * b.x) - (a.x * b.z);
        let z = (a.x * b.y) - (a.y * b.x);
        Vec3::new(x, y, z)
    }

    pub fn dot(a: Vec3, b: Vec3) -> f32 {
        (a.x * b.x) + (a.y * b.y) + (a.z * b.z)
    }

    fn length_squared(a: Vec3) -> f32 {
        Vec3::dot(a, a)
    }

    pub fn length(a: Vec3) -> f32 {
        Vec3::length_squared(a).sqrt()
    }

    pub fn normalize(a: Vec3) -> Vec3 {
        let length = Vec3::length(a);
        a / length
    }
}

// -----------------------------------------------------------------------------------------
// Helpers
impl Vec3 {
    #[allow(dead_code)]
    pub fn print(a: Vec3, label: &str) {
        println!("{} = [{:.2}, {:.2}, {:.2}]", label, a.x, a.y, a.z);
    }
}

// -----------------------------------------------------------------------------------------
