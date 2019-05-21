// -----------------------------------------------------------------------------------------

mod raytrace;
use raytrace::camera::Camera;
use raytrace::camera::Tracer;
use raytrace::ray::Ray;
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

fn sample_background(ray: &Ray, colour_bottom: Vec3, colour_top: Vec3) -> Vec3 {
    let t = (ray.direction.y + 1.0) * 0.5;
    Vec3::lerp(colour_bottom, colour_top, t)
}

// -----------------------------------------------------------------------------------------

fn sample_scene(ray: &Ray) -> Vec3 {
    // Setup colours
    let background_colour_bottom = Vec3::new(1.0, 1.0, 1.0);
    let background_colour_top = Vec3::new(0.5, 0.7, 1.0);

    // Test against spheres
    let sphere_position = Vec3::ZERO;
    let (hit_sphere, _hit_distance, hit_position) =
        raytrace::intersect::ray_sphere(&ray, sphere_position, 3.0);
    let mut hit_normal = Vec3::normalize(hit_position - sphere_position);
    hit_normal.x = (hit_normal.x + 1.0) * 0.5;
    hit_normal.y = (hit_normal.y + 1.0) * 0.5;
    hit_normal.z = (hit_normal.z + 1.0) * 0.5;

    // Test against plane?
    let mut hit_plane = false;
    if !hit_sphere {
        let (hit_test, _hit_distance, _hit_position) =
            raytrace::intersect::ray_plane(&ray, Vec3::ZERO, Vec3::UP);
        hit_plane = hit_test;
    }

    // Shade pixel
    if hit_sphere {
        hit_normal
    } else if hit_plane {
        Vec3::new(0.2, 0.2, 0.4)
    } else {
        sample_background(&ray, background_colour_bottom, background_colour_top)
    }
}

// -----------------------------------------------------------------------------------------

fn pixel_from_vector(v: Vec3) -> bmp::Pixel {
    let r = (v.x * 255.0) as u8;
    let g = (v.y * 255.0) as u8;
    let b = (v.z * 255.0) as u8;
    bmp::Pixel::new(r, g, b)
}

// -----------------------------------------------------------------------------------------

fn draw_scene(image: &mut bmp::Image, image_width: u32, image_height: u32) {
    // Setup camera
    let camera = Camera::new(Vec3::new(0.0, 1.0, -10.0), Vec3::ZERO, 90.0);
    let tracer = Tracer::new(&camera, image_width, image_height);

    // For each pixel...
    for pixel_x in 0..image_width {
        for pixel_y in 0..image_height {
            // Generate ray
            let ray = tracer.get_ray(pixel_x, pixel_y);

            // Sample scene
            let colour = sample_scene(&ray);

            // Write pixel
            let pixel = pixel_from_vector(colour);
            image.set_pixel(pixel_x, image_height - pixel_y - 1, pixel);
        }
    }
}

// -----------------------------------------------------------------------------------------
