// -----------------------------------------------------------------------------------------

mod raytrace;
use raytrace::camera::Camera;
use raytrace::camera::Tracer;
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
    let tracer = Tracer::new(&camera, image_width, image_height);

    // For each pixel...
    let mut pixel = bmp::Pixel::new(0, 0, 0);
    for pixel_x in 0..image_width {
        for pixel_y in 0..image_height {
            // Generate ray
            let ray = tracer.get_ray(pixel_x, pixel_y);

            // Test against scenes
            let (hit, _hit_distance, _hit_position) =
                raytrace::intersect::ray_sphere(ray, Vec3::ZERO, 3.0);

            // Shade pixel
            pixel.r = if hit { 255 } else { 0 };
            image.set_pixel(pixel_x, image_height - pixel_y - 1, pixel);
        }
    }
}

// -----------------------------------------------------------------------------------------
