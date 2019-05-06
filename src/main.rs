// -----------------------------------------------------------------------------------------

type Vec3 = [f32; 3];

// -----------------------------------------------------------------------------------------

fn main() {
    // Setup image
    const IMAGE_WIDTH: u32 = 640;
    const IMAGE_HEIGHT: u32 = 480;
    let mut image = bmp::Image::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    // Clear image
    clear_image(&mut image, IMAGE_WIDTH, IMAGE_HEIGHT);

    // Draw scene
    draw_scene(&mut image, IMAGE_WIDTH, IMAGE_HEIGHT);

    // Save image
    save_image(&image, "output.bmp");
}

// -----------------------------------------------------------------------------------------

fn clear_image(image: &mut bmp::Image, image_width: u32, image_height: u32) {
    let mut pixel = bmp::Pixel::new(0, 0, 0);
    for x in 0..image_width {
        for y in 0..image_height {
            pixel.r = ((x as f32 / image_width as f32) * 255.0) as u8;
            pixel.g = ((y as f32 / image_height as f32) * 255.0) as u8;
            image.set_pixel(x, y, pixel);
        }
    }
}

// -----------------------------------------------------------------------------------------

fn save_image(image: &bmp::Image, filename: &str) {
    image.save(filename).expect("Failed");
}

// -----------------------------------------------------------------------------------------

fn draw_scene(image: &mut bmp::Image, image_width: u32, image_height: u32) {
    // Camera config
    let camera_position: Vec3 = [0.0, 10.0, -10.0];
    let camera_lookat: Vec3 = [0.0, 0.0, 0.0];
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
    let camera_forward = vec3_normalize(vec3_subtract(camera_lookat, camera_position));
    let camera_right = vec3_normalize(vec3_cross([0.0, 1.0, 0.0], camera_forward));
    let camera_up = vec3_normalize(vec3_cross(camera_forward, camera_right));

    // Print camera basis
    vec3_print(camera_right, "Camera right");
    vec3_print(camera_up, "Camera up");
    vec3_print(camera_forward, "Camera forwards");

    // Calculate frustum extents
    let frustum_near_extents: Vec3 = vec3_add(
        vec3_multiply_scalar(camera_right, frustum_near_half_width),
        vec3_multiply_scalar(camera_up, frustum_near_half_height),
    );
    let frustum_far_extents: Vec3 = vec3_add(
        vec3_multiply_scalar(camera_right, frustum_far_half_width),
        vec3_multiply_scalar(camera_up, frustum_far_half_height),
    );

    // Calculate frustum bottom left corners
    let frustum_near_bottom_left: Vec3 = vec3_subtract(
        vec3_add(
            camera_position,
            vec3_multiply_scalar(camera_forward, camera_near),
        ),
        frustum_near_extents,
    );
    let frustum_far_bottom_left: Vec3 = vec3_subtract(
        vec3_add(
            camera_position,
            vec3_multiply_scalar(camera_forward, camera_far),
        ),
        frustum_far_extents,
    );

    // For each pixel...
    let mut pixel = bmp::Pixel::new(0, 0, 0);
    for pixel_x in 0..image_width {
        let perc_x = pixel_x as f32 / image_width as f32;
        for pixel_y in 0..image_height {
            let perc_y = pixel_y as f32 / image_height as f32;
            let near_offset = vec3_add(
                vec3_multiply_scalar(camera_right, frustum_near_width * perc_x),
                vec3_multiply_scalar(camera_up, frustum_near_height * perc_y),
            );
            let far_offset = vec3_add(
                vec3_multiply_scalar(camera_right, frustum_far_width * perc_x),
                vec3_multiply_scalar(camera_up, frustum_far_height * perc_y),
            );

            // Calculate ray
            let ray_start = vec3_add(frustum_near_bottom_left, near_offset);
            let ray_end = vec3_add(frustum_far_bottom_left, far_offset);
            let ray_dir = vec3_normalize(vec3_subtract(ray_end, ray_start));

            // Test against scene
            pixel.r = if intersect_sphere(ray_start, ray_dir, [0.0, 0.0, 0.0], 1.0) > 0.0 {
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
    let m: Vec3 = vec3_subtract(p, sc);
    let b: f32 = vec3_dot(m, d);
    let c: f32 = vec3_dot(m, m) - sr * sr;

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
    let mut t: f32 = -b - discr.sqrt();

    // If t is negative, ray started inside sphere so clamp t to zero
    if t < 0.0 {
        t = 0.0;
    }
    //q = p + t * d;

    return 1.0;
}

// -----------------------------------------------------------------------------------------

fn vec3_add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

// -----------------------------------------------------------------------------------------

fn vec3_subtract(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

// -----------------------------------------------------------------------------------------

fn vec3_multiply(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

// -----------------------------------------------------------------------------------------

fn vec3_multiply_scalar(a: Vec3, b: f32) -> Vec3 {
    [a[0] * b, a[1] * b, a[2] * b]
}

// -----------------------------------------------------------------------------------------

fn vec3_length_squared(a: Vec3) -> f32 {
    (a[0] * a[0]) + (a[1] * a[1]) + (a[2] * a[2])
}

// -----------------------------------------------------------------------------------------

fn vec3_length(a: Vec3) -> f32 {
    vec3_length_squared(a).sqrt()
}
// -----------------------------------------------------------------------------------------

fn vec3_normalize(a: Vec3) -> Vec3 {
    let length = vec3_length(a);
    [a[0] / length, a[1] / length, a[2] / length]
}

// -----------------------------------------------------------------------------------------

fn vec3_cross(a: Vec3, b: Vec3) -> Vec3 {
    let x = (a[1] * b[2]) - (a[2] * b[1]);
    let y = (a[2] * b[0]) - (a[0] * b[2]);
    let z = (a[0] * b[1]) - (a[1] * b[0]);
    [x, y, z]
}

// -----------------------------------------------------------------------------------------

fn vec3_dot(a: Vec3, b: Vec3) -> f32 {
    (a[0] * b[0]) + (a[1] * b[1]) + (a[2] * b[2])
}

// -----------------------------------------------------------------------------------------

fn vec3_print(a: Vec3, label: &str) {
    println!("{} = [{:.2}, {:.2}, {:.2}]", label, a[0], a[1], a[2]);
}

// -----------------------------------------------------------------------------------------
