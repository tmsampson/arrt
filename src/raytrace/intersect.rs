// -----------------------------------------------------------------------------------------

use super::ray::Ray;
use super::vector::Vec3;

// -----------------------------------------------------------------------------------------

pub fn ray_sphere(ray: Ray, sc: Vec3, sr: f32) -> (bool, f32, Vec3) {
    const NO_INTERSECTION: (bool, f32, Vec3) = (false, 0.0, Vec3::ZERO);

    let m: Vec3 = ray.origin - sc;
    let b: f32 = Vec3::dot(m, ray.direction);
    let c: f32 = Vec3::dot(m, m) - sr * sr;

    // Exit if r’s origin outside s (c > 0) and r pointing away from s (b > 0)
    if c > 0.0 && b > 0.0 {
        return NO_INTERSECTION;
    }
    let discr: f32 = b * b - c;

    // A negative discriminant corresponds to ray missing sphere
    if discr < 0.0 {
        return NO_INTERSECTION;
    }

    // Ray now found to intersect sphere, compute smallest t value of intersection
    let mut hit_distance: f32 = -b - discr.sqrt();

    // If t is negative, ray started inside sphere so clamp t to zero
    if hit_distance < 0.0 {
        hit_distance = 0.0;
    }

    // Return valid hit
    let hit_position: Vec3 = ray.origin + (ray.direction * hit_distance);
    return (true, hit_distance, hit_position);
}

// -----------------------------------------------------------------------------------------
