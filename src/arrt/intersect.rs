// -----------------------------------------------------------------------------------------

use super::geometry::Plane;
use super::geometry::Sphere;
use super::ray::Ray;
use super::ray::RayHitResult;
use super::vector::Vec3;

// -----------------------------------------------------------------------------------------

pub fn ray_plane(ray: &Ray, plane: &Plane) -> RayHitResult {
    const TOLLERANCE: f32 = 0.001;
    let denom = Vec3::dot(plane.normal, ray.direction);
    if denom.abs() > TOLLERANCE {
        let plane_to_ray = plane.position - ray.origin;
        let t = Vec3::dot(plane_to_ray, plane.normal) / denom;
        let hit_pos = ray.get_point(t);
        //let row = (hit_pos.z * 0.25).round().abs() % 2.0;
        //let col = (hit_pos.x * 0.25).round().abs() % 2.0;
        //let pick = (row + col) % 2.0;
        //let diffuse = plane.diffuse; // Vec3::lerp(plane.diffuse, plane.diffuse * 0.5, pick);
        RayHitResult::new(t > TOLLERANCE, t, hit_pos, plane.normal, "mirror")
    } else {
        RayHitResult::NO_HIT
    }
}

// -----------------------------------------------------------------------------------------

pub fn ray_sphere(ray: &Ray, sphere: &Sphere) -> RayHitResult {
    let m: Vec3 = ray.origin - sphere.centre;
    let b: f32 = Vec3::dot(m, ray.direction);
    let c: f32 = Vec3::dot(m, m) - (sphere.radius * sphere.radius);

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
    const TOLLERANCE: f32 = 0.001;
    let hit_position: Vec3 = ray.origin + (ray.direction * hit_distance);
    RayHitResult::new(
        hit_distance > TOLLERANCE,
        hit_distance,
        hit_position,
        Vec3::UP,
        sphere.material,
    )
}

// -----------------------------------------------------------------------------------------
