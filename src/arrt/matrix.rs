// -----------------------------------------------------------------------------------------
// Useful reference: http://glmatrix.net/docs/mat4.js.html#line738
// -----------------------------------------------------------------------------------------

use super::vector::Vec3;
use serde::{Deserialize, Serialize};
use std::ops;

// -----------------------------------------------------------------------------------------
// Type
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Mat4 {
    pub rows: [[f32; 4]; 4],
}

// -----------------------------------------------------------------------------------------
// Constants
impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        rows: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    const EPSILON: f32 = 0.001;
}

// -----------------------------------------------------------------------------------------
// Helpers
impl Mat4 {
    pub fn make_rotation(axis: Vec3, angle: f32) -> Mat4 {
        let len = Vec3::length(axis);
        if len < Mat4::EPSILON {
            return Mat4::IDENTITY;
        }
        let recip = 1.0 / len;
        let (x, y, z) = (axis.x * recip, axis.y * recip, axis.z * recip);
        let rad = angle.to_radians();
        let s = rad.sin();
        let c = rad.cos();
        let t = 1.0 - c;
        Mat4 {
            rows: [
                [x * x * t + c, y * x * t + z * s, z * x * t - y * s, 0.0],
                [x * y * t - z * s, y * y * t + c, z * y * t + x * s, 0.0],
                [x * z * t + y * s, y * z * t - x * s, z * z * t + c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn make_rotation_yaxis(angle: f32) -> Mat4 {
        let rad = angle.to_radians();
        let s = rad.sin();
        let c = rad.cos();
        Mat4 {
            rows: [
                [c, 0.0, -s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

// -----------------------------------------------------------------------------------------
// Operators
impl ops::Mul<Vec3> for Mat4 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        let x = (self.rows[0][0] * rhs.x)
            + (self.rows[0][1] * rhs.y)
            + (self.rows[0][2] * rhs.z)
            + self.rows[0][3];
        let y = (self.rows[1][0] * rhs.x)
            + (self.rows[1][1] * rhs.y)
            + (self.rows[1][2] * rhs.z)
            + self.rows[1][3];
        let z = (self.rows[2][0] * rhs.x)
            + (self.rows[2][1] * rhs.y)
            + (self.rows[2][2] * rhs.z)
            + self.rows[2][3];
        Vec3::new(x, y, z)
    }
}
