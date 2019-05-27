// -----------------------------------------------------------------------------------------

mod raytrace;
use raytrace::camera::Camera;
use raytrace::camera::Tracer;
use raytrace::ray::Ray;
use raytrace::vector::Vec3;

// -----------------------------------------------------------------------------------------

use rand::prelude::*;

// -----------------------------------------------------------------------------------------

// use std::thread;
// fn main() {
//     const STACK_SIZE: usize = 40 * 1024 * 1024;

//     // Spawn thread with explicit stack size
//     let child = thread::Builder::new()
//         .name("raytracer".into())
//         .stack_size(STACK_SIZE)
//         .spawn(run)
//         .unwrap();

//     // Wait for thread to join
//     child.join().unwrap();
// }

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

fn draw_scene(image: &mut bmp::Image, image_width: u32, image_height: u32) {
    // Setup camera
    let camera_position = Vec3::new(0.0, 1.0, 3.0);
    let camera_lookat = Vec3::new(0.0, 1.0, 0.0);
    let camera = Camera::new(camera_position, camera_lookat, 90.0);
    let tracer = Tracer::new(&camera, image_width, image_height);

    // Setup RNG
    let mut rng: StdRng = SeedableRng::seed_from_u64(0);

    // Generate random sampling offsets
    const SAMPLES_PER_PIXEL: usize = 8;
    const ADDITIONAL_SAMPLES: usize = SAMPLES_PER_PIXEL - 1;
    let mut sample_offsets_x: [f32; ADDITIONAL_SAMPLES] = [0.0; ADDITIONAL_SAMPLES];
    let mut sample_offsets_y: [f32; ADDITIONAL_SAMPLES] = [0.0; ADDITIONAL_SAMPLES];
    for sample_index in 0..ADDITIONAL_SAMPLES {
        sample_offsets_x[sample_index] = rng.gen();
        sample_offsets_y[sample_index] = rng.gen();
    }

    // For each pixel...
    let mut pixel = bmp::Pixel::new(0, 0, 0);
    for pixel_x in 0..image_width {
        for pixel_y in 0..image_height {
            let pixel_x_f = pixel_x as f32;
            let pixel_y_f = pixel_y as f32;

            // Sample centroid
            let ray_centroid = tracer.get_ray(pixel_x_f, pixel_y_f);
            let mut colour = sample_scene(&ray_centroid, &mut rng);

            // Take additional samples
            for sample_index in 0..ADDITIONAL_SAMPLES {
                let offset_x = (sample_offsets_x[sample_index] - 0.5) * 0.99;
                let offset_y = (sample_offsets_y[sample_index] - 0.5) * 0.99;
                let ray = tracer.get_ray(pixel_x_f + offset_x, pixel_y_f + offset_y);
                colour += sample_scene(&ray, &mut rng);
            }

            // Average samples and store in pixel
            colour /= SAMPLES_PER_PIXEL as f32;
            Vec3::copy_to_pixel(colour, &mut pixel);

            // Write pixel
            image.set_pixel(pixel_x, image_height - pixel_y - 1, pixel);
        }
    }
}

// -----------------------------------------------------------------------------------------

fn sample_scene(ray: &Ray, rng: &mut StdRng) -> Vec3 {
    // Setup colours
    let background_colour_bottom = Vec3::new(1.0, 1.0, 1.0);
    let background_colour_top = Vec3::new(0.5, 0.7, 1.0);

    // Test against spheres
    let sphere_position = Vec3::new(0.0, 1.0, 0.0);
    let sphere_radius = 1.0;
    let (hit_sphere, hit_distance, hit_position) =
        raytrace::intersect::ray_sphere(&ray, sphere_position, sphere_radius);
    let hit_normal = Vec3::normalize(hit_position - sphere_position);

    // Generate colour from normal
    let _hit_normal_colour = Vec3::new(
        (hit_normal.x + 1.0) * 0.5,
        (hit_normal.y + 1.0) * 0.5,
        (hit_normal.z + 1.0) * 0.5,
    );

    // Test against plane?
    let (hit_test_p, hit_distance_p, hit_position_p) =
        raytrace::intersect::ray_plane(&ray, Vec3::ZERO, Vec3::UP);
    if hit_test_p && (!hit_sphere || (hit_distance_p < hit_distance)) {
        let hit_normal_p = Vec3::UP;

        let absorbed = 0.5;
        let reflected = 1.0 - absorbed;
        let point = hit_position_p + hit_normal_p + Vec3::random_point_in_unit_sphere(rng);
        let reflected_ray = Ray::new(
            hit_position_p + (hit_normal_p * 0.00001),
            Vec3::normalize(point - hit_position_p),
        );
        return sample_scene(&reflected_ray, rng) * reflected;
    }

    // Shade pixel
    if hit_sphere {
        let absorbed = 0.5;
        let reflected = 1.0 - absorbed;
        let point = hit_position + hit_normal + Vec3::random_point_in_unit_sphere(rng);
        let reflected_ray = Ray::new(
            hit_position + (hit_normal * 0.00001),
            Vec3::normalize(point - hit_position),
        );
        sample_scene(&reflected_ray, rng) * reflected
    } else {
        sample_background(&ray, background_colour_bottom, background_colour_top)
    }
}

// -----------------------------------------------------------------------------------------
