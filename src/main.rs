// -----------------------------------------------------------------------------------------

mod vectorlib;
use vectorlib::Vec3;

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
    // Camera config
    let camera_position = Vec3::new(0.0, 10.0, -10.0);
    let camera_lookat = Vec3::new(0.0, 0.0, 0.0);
    let camera_aspect = image_width as f32 / image_height as f32;
    let camera_fov: f32 = 90.0;
    let camera_near = 1.0;
    let camera_far = 10000.0;

    // Calculate frustum
    let frustum_mult = (camera_fov * 0.5).tan();
    let frustum_near_width = camera_near * frustum_mult * 2.0;
    let frustum_near_height = frustum_near_width / camera_aspect;
    let frustum_near_half_width = frustum_near_width * 0.5;
    let frustum_near_half_height = frustum_near_height * 0.5;
    let frustum_far_width = camera_far * frustum_mult * 2.0;
    let frustum_far_height = frustum_far_width / camera_aspect;
    let frustum_far_half_width = frustum_far_width * 0.5;
    let frustum_far_half_height = frustum_far_height * 0.5;

    // Calculate camera basis
    let camera_forward = Vec3::normalize(camera_lookat - camera_position);
    let camera_right = Vec3::normalize(Vec3::cross(Vec3::UP, camera_forward));
    let camera_up = Vec3::normalize(Vec3::cross(camera_forward, camera_right));

    // Print camera basis
    Vec3::print(camera_right, "Camera right");
    Vec3::print(camera_up, "Camera up");
    Vec3::print(camera_forward, "Camera forwards");

    // Calculate frustum extents
    let frustum_near_extents: Vec3 =
        (camera_right * frustum_near_half_width) + (camera_up * frustum_near_half_height);
    let frustum_far_extents: Vec3 =
        (camera_right * frustum_far_half_width) + (camera_up * frustum_far_half_height);

    // Calculate frustum bottom left corners
    let frustum_near_bottom_left: Vec3 =
        (camera_position + (camera_forward * camera_near)) - frustum_near_extents;
    let frustum_far_bottom_left: Vec3 =
        (camera_position + (camera_forward * camera_far)) - frustum_far_extents;

    // For each pixel...
    let mut pixel = bmp::Pixel::new(0, 0, 0);
    for pixel_x in 0..image_width {
        let perc_x = pixel_x as f32 / image_width as f32;
        for pixel_y in 0..image_height {
            let perc_y = pixel_y as f32 / image_height as f32;
            let near_offset = (camera_right * frustum_near_width * perc_x)
                + (camera_up * frustum_near_height * perc_y);
            let far_offset = (camera_right * frustum_far_width * perc_x)
                + (camera_up * frustum_far_height * perc_y);

            // Calculate ray
            let ray_start = frustum_near_bottom_left + near_offset;
            let ray_end = frustum_far_bottom_left + far_offset;
            let ray_dir = Vec3::normalize(ray_end - ray_start);

            // Test against scene
            pixel.r = if intersect_sphere(ray_start, ray_dir, Vec3::ZERO, 3.0) > 0.0 {
                255
            } else {
                0
            };
            image.set_pixel(pixel_x, image_height - pixel_y - 1, pixel);
        }
    }
}

// -----------------------------------------------------------------------------------------

fn intersect_sphere(p: Vec3, d: Vec3, sc: Vec3, sr: f32) -> f32 {
    let m: Vec3 = p - sc;
    let b: f32 = Vec3::dot(m, d);
    let c: f32 = Vec3::dot(m, m) - sr * sr;

    // Exit if r’s origin outside s (c > 0) and r pointing away from s (b > 0)
    if c > 0.0 && b > 0.0 {
        return 0.0;
    }
    let discr: f32 = b * b - c;

    // A negative discriminant corresponds to ray missing sphere
    if discr < 0.0 {
        return 0.0;
    }

    // Ray now found to intersect sphere, compute smallest t value of intersection
    //let mut t: f32 = -b - discr.sqrt();

    // If t is negative, ray started inside sphere so clamp t to zero
    //if t < 0.0 {
    //    t = 0.0;
    //}
    //q = p + t * d;

    return 1.0;
}

// -----------------------------------------------------------------------------------------
