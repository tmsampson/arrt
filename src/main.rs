// -----------------------------------------------------------------------------------------

mod raytrace;
use raytrace::camera::Camera;
use raytrace::camera::Tracer;
use raytrace::ray::Ray;
use raytrace::ray::RayHitResult;
use raytrace::vector::Vec3;

// -----------------------------------------------------------------------------------------

use rand::prelude::*;

// -----------------------------------------------------------------------------------------
// Config | Image
const IMAGE_FILENAME: &str = "output.bmp";
const IMAGE_WIDTH: u32 = 640;
const IMAGE_HEIGHT: u32 = 480;

// -----------------------------------------------------------------------------------------
// Config | Rendering
const RNG_SEED: u64 = 0;
const SAMPLES_PER_PIXEL: usize = 8;

// -----------------------------------------------------------------------------------------
// Config | Camera
const CAMERA_FOV: f32 = 90.0;
const CAMERA_POSITION: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 3.0,
};
const CAMERA_LOOKAT: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

// -----------------------------------------------------------------------------------------
// Config | Sky
const SKY_COLOUR_BOTTOM: Vec3 = Vec3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};
const SKY_COLOUR_TOP: Vec3 = Vec3 {
    x: 0.5,
    y: 0.7,
    z: 1.0,
};

// -----------------------------------------------------------------------------------------
// Config | Debug
const DEBUG_NORMALS: bool = false;
const DEBUG_SHOW_PROGRESS: bool = false;

// -----------------------------------------------------------------------------------------

fn main() {
    // Setup image
    let mut image = bmp::Image::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    // Draw scene
    draw_scene(&mut image);

    // Save image
    save_image(&image, IMAGE_FILENAME);
}

// -----------------------------------------------------------------------------------------

fn save_image(image: &bmp::Image, filename: &str) {
    image.save(filename).expect("Failed");
}

// -----------------------------------------------------------------------------------------

fn sample_background(ray: &Ray) -> Vec3 {
    let t = (ray.direction.y + 1.0) * 0.5;
    Vec3::lerp(SKY_COLOUR_BOTTOM, SKY_COLOUR_TOP, t)
}

// -----------------------------------------------------------------------------------------

fn draw_scene(image: &mut bmp::Image) {
    // Setup camera
    let image_width = image.get_width();
    let image_height = image.get_height();
    let camera = Camera::new(CAMERA_POSITION, CAMERA_LOOKAT, CAMERA_FOV);
    let tracer = Tracer::new(&camera, image_width, image_height);

    // Setup RNG
    let mut rng: StdRng = SeedableRng::seed_from_u64(RNG_SEED);

    // Generate random sampling offsets
    const ADDITIONAL_SAMPLES: usize = SAMPLES_PER_PIXEL - 1;
    let mut sample_offsets_x: [f32; ADDITIONAL_SAMPLES] = [0.0; ADDITIONAL_SAMPLES];
    let mut sample_offsets_y: [f32; ADDITIONAL_SAMPLES] = [0.0; ADDITIONAL_SAMPLES];
    for sample_index in 0..ADDITIONAL_SAMPLES {
        sample_offsets_x[sample_index] = rng.gen();
        sample_offsets_y[sample_index] = rng.gen();
    }

    // For each scanline...
    let mut pixel = bmp::Pixel::new(0, 0, 0);
    for pixel_y in 0..image_height {
        let pixel_y_f = pixel_y as f32;

        // Show progress?
        if DEBUG_SHOW_PROGRESS {
            println!("Tracing scanline {} / {}", pixel_y + 1, image_height);
        }

        // For each column
        for pixel_x in 0..image_width {
            let pixel_x_f = pixel_x as f32;

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
    let mut result = RayHitResult::MAX_HIT;

    // Test against spheres
    let sphere_result = sample_scene_spheres(ray);
    if sphere_result.hit {
        result = sphere_result;
    }

    // Test against planes
    let plane_result = sample_scene_planes(ray);
    if plane_result.hit && (plane_result.distance < result.distance) {
        result = plane_result;
    }

    // Use background?
    if !result.hit {
        return sample_background(ray);
    }

    // Debug normals?
    if DEBUG_NORMALS {
        return Vec3::new(
            (result.normal.x + 1.0) * 0.5,
            (result.normal.y + 1.0) * 0.5,
            (result.normal.z + 1.0) * 0.5,
        );
    }

    // Shade pixel (diffuse)
    let absorbed = 0.3;
    let reflected = 1.0 - absorbed;
    let refelcted_point = result.position + result.normal + Vec3::random_point_in_unit_sphere(rng);
    let reflected_ray_origin = result.position + (result.normal * 0.00001);
    let refelcted_ray_direction = Vec3::normalize(refelcted_point - result.position);
    let reflected_ray = Ray::new(reflected_ray_origin, refelcted_ray_direction);
    sample_scene(&reflected_ray, rng) * reflected
}

// -----------------------------------------------------------------------------------------

fn sample_scene_spheres(ray: &Ray) -> RayHitResult {
    let sphere_position = Vec3::new(0.0, 1.0, 0.0);
    let sphere_radius = 1.0;

    let mut result = raytrace::intersect::ray_sphere(&ray, sphere_position, sphere_radius);
    if result.hit {
        result.normal = Vec3::normalize(result.position - sphere_position);
        result
    } else {
        RayHitResult::NO_HIT
    }
}

// -----------------------------------------------------------------------------------------

fn sample_scene_planes(ray: &Ray) -> RayHitResult {
    raytrace::intersect::ray_plane(&ray, Vec3::ZERO, Vec3::UP)
}

// -----------------------------------------------------------------------------------------
