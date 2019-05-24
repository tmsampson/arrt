// -----------------------------------------------------------------------------------------

mod raytrace;
use raytrace::camera::Camera;
use raytrace::camera::Tracer;
use raytrace::ray::Ray;
use raytrace::vector::Vec3;

// -----------------------------------------------------------------------------------------

use rand::prelude::*;

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
    let camera = Camera::new(Vec3::new(0.0, 1.0, 10.0), Vec3::ZERO, 90.0);
    let tracer = Tracer::new(&camera, image_width, image_height);

    // Setup sample offsets
    const SAMPLES_PER_PIXEL: usize = 8;
    const ADDITIONAL_SAMPLES: usize = SAMPLES_PER_PIXEL - 1;
    let mut rng = rand::thread_rng();
    let mut sample_offsets_x: [f32; ADDITIONAL_SAMPLES] = [0.0; ADDITIONAL_SAMPLES];
    let mut sample_offsets_y: [f32; ADDITIONAL_SAMPLES] = [0.0; ADDITIONAL_SAMPLES];
    for sample_index in 0..ADDITIONAL_SAMPLES {
        sample_offsets_x[sample_index] = rng.gen();
        sample_offsets_y[sample_index] = rng.gen();
    }

    // For each pixel...
    for pixel_x in 0..image_width {
        for pixel_y in 0..image_height {
            let pixel_x_f = pixel_x as f32;
            let pixel_y_f = pixel_y as f32;

            // Sample centroid
            let ray_centroid = tracer.get_ray(pixel_x_f, pixel_y_f);
            let mut colour = sample_scene(&ray_centroid);

            // Take additional samples
            for sample_index in 0..ADDITIONAL_SAMPLES {
                let offset_x = (sample_offsets_x[sample_index] - 0.5) * 0.99;
                let offset_y = (sample_offsets_y[sample_index] - 0.5) * 0.99;
                let ray = tracer.get_ray(pixel_x_f + offset_x, pixel_y_f + offset_y);
                colour = colour + sample_scene(&ray);
            }

            // Write pixel
            let pixel = pixel_from_vector(colour / (SAMPLES_PER_PIXEL as f32));
            image.set_pixel(pixel_x, image_height - pixel_y - 1, pixel);
        }
    }
}

// -----------------------------------------------------------------------------------------
