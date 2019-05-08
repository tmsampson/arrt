// -----------------------------------------------------------------------------------------

mod raytrace;
use raytrace::camera::Camera;
use raytrace::vector::Vec3;

// -----------------------------------------------------------------------------------------

fn main() {
    // Setup image
    const IMAGE_WIDTH: u32 = 640;
    const IMAGE_HEIGHT: u32 = 480;
    let mut image = bmp::Image::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    // Draw scene
    draw_scene(&mut image, IMAGE_WIDTH, IMAGE_HEIGHT);

    // Save image
    save_image(&image, "output.bmp");
}

// -----------------------------------------------------------------------------------------

fn save_image(image: &bmp::Image, filename: &str) {
    image.save(filename).expect("Failed");
}

// -----------------------------------------------------------------------------------------

fn draw_scene(image: &mut bmp::Image, image_width: u32, image_height: u32) {
    // Setup camera
    let camera = Camera::new(Vec3::new(0.0, 10.0, -10.0), Vec3::ZERO, 90.0);

    // Camera config
    let camera_aspect = image_width as f32 / image_height as f32;

    // Calculate frustum
    let frustum_mult = (camera.fov * 0.5).tan();
    let frustum_near_half_width = camera.near_distance * frustum_mult;
    let frustum_near_half_height = frustum_near_half_width / camera_aspect;
    let frustum_far_half_width = camera.far_distance * frustum_mult;
    let frustum_far_half_height = frustum_far_half_width / camera_aspect;

    // Calculate frustum extents
    let frustum_near_extents: Vec3 =
        (camera.right * frustum_near_half_width) + (camera.up * frustum_near_half_height);
    let frustum_far_extents: Vec3 =
        (camera.right * frustum_far_half_width) + (camera.up * frustum_far_half_height);

    // Calculate frustum bottom left corners
    let frustum_near_bottom_left: Vec3 =
        (camera.position + (camera.forward * camera.near_distance)) - frustum_near_extents;
    let frustum_far_bottom_left: Vec3 =
        (camera.position + (camera.forward * camera.far_distance)) - frustum_far_extents;

    // For each pixel...
    let mut pixel = bmp::Pixel::new(0, 0, 0);
    for pixel_x in 0..image_width {
        let perc_x = pixel_x as f32 / image_width as f32;
        for pixel_y in 0..image_height {
            let perc_y = pixel_y as f32 / image_height as f32;

            // Calculate ray
            let ray_start = frustum_near_bottom_left + (camera.right * frustum_near_half_width * 2.0 * perc_x)
                + (camera.up * frustum_near_half_height * 2.0 * perc_y);
            let ray_end = frustum_far_bottom_left + (camera.right * frustum_far_half_width * 2.0 * perc_x)
                + (camera.up * frustum_far_half_height * 2.0 * perc_y);
            let ray_dir = Vec3::normalize(ray_end - ray_start);

            // Test against scene
            let (hit, _hit_distance, _hit_position) =
                intersect_sphere(ray_start, ray_dir, Vec3::ZERO, 3.0);

            // Shade pixel
            pixel.r = if hit { 255 } else { 0 };
            image.set_pixel(pixel_x, image_height - pixel_y - 1, pixel);
        }
    }
}

// -----------------------------------------------------------------------------------------

fn intersect_sphere(p: Vec3, d: Vec3, sc: Vec3, sr: f32) -> (bool, f32, Vec3) {
    const NO_INTERSECTION: (bool, f32, Vec3) = (false, 0.0, Vec3::ZERO);

    let m: Vec3 = p - sc;
    let b: f32 = Vec3::dot(m, d);
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
    let hit_position: Vec3 = p + (d * hit_distance);
    return (true, hit_distance, hit_position);
}

// -----------------------------------------------------------------------------------------
