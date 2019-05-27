// -----------------------------------------------------------------------------------------

use super::ray::Ray;
use super::ray::RayHitResult;
use super::vector::Vec3;

// -----------------------------------------------------------------------------------------

pub fn ray_plane(ray: &Ray, plane_position: Vec3, plane_normal: Vec3) -> RayHitResult {
    const TOLLERANCE: f32 = 0.00001;

    let denom = Vec3::dot(plane_normal, ray.direction);
    if denom.abs() > TOLLERANCE {
        let plane_to_ray = plane_position - ray.origin;
        let t = Vec3::dot(plane_to_ray, plane_normal) / denom;
        return RayHitResult::new(t > TOLLERANCE, t, ray.get_point(t), plane_normal);
    } else {
        return RayHitResult::NO_HIT;
    }
}

// -----------------------------------------------------------------------------------------

pub fn ray_sphere(ray: &Ray, sphere_position: Vec3, sphere_radius: f32) -> RayHitResult {
    let m: Vec3 = ray.origin - sphere_position;
    let b: f32 = Vec3::dot(m, ray.direction);
    let c: f32 = Vec3::dot(m, m) - sphere_radius * sphere_radius;

    // Exit if r’s origin outside s (c > 0) and r pointing away from s (b > 0)
    if c > 0.0 && b > 0.0 {
        return RayHitResult::NO_HIT;
    }
    let discr: f32 = b * b - c;

    // A negative discriminant corresponds to ray missing sphere
    if discr < 0.0 {
        return RayHitResult::NO_HIT;
    }

    // Ray now found to intersect sphere, compute smallest t value of intersection
    let mut hit_distance: f32 = -b - discr.sqrt();

    // If t is negative, ray started inside sphere so clamp t to zero
    if hit_distance < 0.0 {
        hit_distance = 0.0;
    }

    // Return valid hit
    let hit_position: Vec3 = ray.origin + (ray.direction * hit_distance);
    RayHitResult::new(true, hit_distance, hit_position, Vec3::UP)
}

// -----------------------------------------------------------------------------------------
