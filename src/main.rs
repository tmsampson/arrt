// -----------------------------------------------------------------------------------------

use clap::{App, Arg};
use rand::prelude::*;
use std::collections::HashMap;

// -----------------------------------------------------------------------------------------

mod raytrace;
use raytrace::camera::Camera;
use raytrace::camera::Tracer;
use raytrace::geometry::Plane;
use raytrace::geometry::Sphere;
use raytrace::misc::StringLiteral;
use raytrace::quality::QualityPreset;
use raytrace::ray::Ray;
use raytrace::ray::RayHitResult;
use raytrace::vector::Vec3;

// -----------------------------------------------------------------------------------------
// Config | Image
const IMAGE_FILENAME: &str = "output.bmp";

// -----------------------------------------------------------------------------------------
// Config | Rendering
const RNG_SEED: u64 = 0;

// -----------------------------------------------------------------------------------------
// Config | Camera
const CAMERA_FOV: f32 = 90.0;
const CAMERA_POSITION: Vec3 = Vec3 {
    x: 0.0,
    y: 10.0,
    z: -20.0,
};
const CAMERA_LOOKAT: Vec3 = Vec3 {
    x: 0.0,
    y: 3.0,
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
// Config | Scene
const SCENE_SPHERES: [Sphere; 3] = [
    Sphere {
        centre: Vec3 {
            x: -6.5,
            y: 3.0,
            z: 0.0,
        },
        radius: 3.0,
        material: "red",
    },
    Sphere {
        centre: Vec3 {
            x: 0.0,
            y: 3.0,
            z: 0.0,
        },
        radius: 3.0,
        material: "mirror",
    },
    Sphere {
        centre: Vec3 {
            x: 6.5,
            y: 3.0,
            z: 0.0,
        },
        radius: 3.0,
        material: "blue",
    },
];

// -----------------------------------------------------------------------------------------

pub struct Material {
    pub diffuse: Vec3,
    pub absorbtion: f32,
}

impl Material {
    pub const NONE: Material = Material {
        diffuse: Vec3::WHITE,
        absorbtion: 1.0,
    };
}

type MaterialTable = HashMap<StringLiteral, Material>;

// -----------------------------------------------------------------------------------------
// Config | Debug
const PROGRESS_UPDATE_INTERVAL: f64 = 1.0;

// -----------------------------------------------------------------------------------------

fn parse_command_line() -> clap::ArgMatches<'static> {
    App::new("Ray Tracer")
        .version("0.0.0")
        .author("Thomas Sampson <tmsampson@gmail.com>")
        .arg(
            Arg::with_name("quality")
                .short("q")
                .long("quality")
                .takes_value(true)
                .help("Quality preset"),
        )
        .arg(
            Arg::with_name("debug-normals")
                .short("normals")
                .long("debug-normals")
                .takes_value(false)
                .help("Debug render normals"),
        )
        .get_matches()
}

// -----------------------------------------------------------------------------------------

struct Job<'a> {
    image: &'a mut bmp::Image,
    quality: QualityPreset,
    rng: &'a mut StdRng
}

// -----------------------------------------------------------------------------------------

fn main() {
    // Start timer
    let timer_begin = time::precise_time_s();

    // Parse command line args
    let args = parse_command_line();
    let quality_preset = args.value_of("quality").unwrap_or("default");
    let quality = raytrace::quality::get_preset(quality_preset.to_string());
    let debug_normals = args.is_present("debug-normals");

    // Setup materials
    let mut materials = MaterialTable::new();
    materials.insert(
        "red",
        Material {
            diffuse: Vec3::RED,
            absorbtion: 0.3,
        },
    );
    materials.insert(
        "green",
        Material {
            diffuse: Vec3::GREEN,
            absorbtion: 0.3,
        },
    );
    materials.insert(
        "blue",
        Material {
            diffuse: Vec3::BLUE,
            absorbtion: 0.3,
        },
    );
    materials.insert(
        "white",
        Material {
            diffuse: Vec3::WHITE,
            absorbtion: 0.3,
        },
    );
    materials.insert(
        "black",
        Material {
            diffuse: Vec3::BLACK,
            absorbtion: 0.3,
        },
    );
    materials.insert(
        "mirror",
        Material {
            diffuse: Vec3::WHITE,
            absorbtion: 0.0,
        },
    );

    let mut rng: StdRng = SeedableRng::seed_from_u64(RNG_SEED);

    let mut job = Job {
        image: &mut bmp::Image::new(quality.image_width, quality.image_height),
        quality,
        rng: &mut rng
    };

    // Draw scene
    draw_scene(&mut job, &materials, debug_normals);

    // Save image
    save_image(&job.image, IMAGE_FILENAME);

    // Stop timer and report
    let timer_end = time::precise_time_s();
    println!("");
    println!("====================================================");
    println!(" SUMMARY");
    println!("====================================================");
    println!("     Output: {}", IMAGE_FILENAME);
    println!("    Quality: {}", quality_preset);
    println!(" Total time: {:.2} seconds", (timer_end - timer_begin));
    println!("====================================================");
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

fn draw_scene(job: &mut Job, materials: &MaterialTable, debug_normals: bool) {
    // Setup camera
    let image_width = job.image.get_width();
    let image_height = job.image.get_height();
    let camera = Camera::new(CAMERA_POSITION, CAMERA_LOOKAT, CAMERA_FOV);
    let tracer = Tracer::new(&camera, image_width, image_height);

    // Generate random sampling offsets
    let additional_samples: usize = job.quality.samples_per_pixel - 1;
    let mut sample_offsets_x = vec![0.0; additional_samples];
    let mut sample_offsets_y = vec![0.0; additional_samples];
    for sample_index in 0..additional_samples {
        sample_offsets_x[sample_index] = job.rng.gen();
        sample_offsets_y[sample_index] = job.rng.gen();
    }

    // Setup regular progress updates
    let mut last_progress_update = time::precise_time_s();

    // For each scanline...
    let mut pixel = bmp::Pixel::new(0, 0, 0);
    for pixel_y in 0..image_height {
        let pixel_y_f = pixel_y as f32;

        // Show progress?
        let now = time::precise_time_s();
        let elapsed = now - last_progress_update;
        if elapsed >= PROGRESS_UPDATE_INTERVAL {
            let row = pixel_y + 1;
            let percent = ((row as f32) / (image_height as f32)) * 100.0;
            println!(
                "Tracing: {:.2}% complete scanline {} / {}",
                percent, row, image_height
            );
            last_progress_update = now;
        }

        // For each column
        for pixel_x in 0..image_width {
            let pixel_x_f = pixel_x as f32;

            // Sample centroid
            let mut bounces = 0;
            let ray_centroid = tracer.get_ray(pixel_x_f, pixel_y_f);
            let mut colour = sample_scene(
                &ray_centroid,
                job,
                &materials,
                &mut bounces,
                job.quality.max_bounces,
                debug_normals,
            );

            // Take additional samples
            for sample_index in 0..additional_samples {
                let mut bounces = 0;
                let offset_x = (sample_offsets_x[sample_index] - 0.5) * 0.99;
                let offset_y = (sample_offsets_y[sample_index] - 0.5) * 0.99;
                let ray = tracer.get_ray(pixel_x_f + offset_x, pixel_y_f + offset_y);
                colour += sample_scene(
                    &ray,
                    job,
                    &materials,
                    &mut bounces,
                    job.quality.max_bounces,
                    debug_normals,
                );
            }

            // Average samples and store in pixel
            colour /= job.quality.samples_per_pixel as f32;
            Vec3::copy_to_pixel(colour, &mut pixel);

            // Write pixel
            job.image
                .set_pixel(pixel_x, image_height - pixel_y - 1, pixel);
        }
    }
}

// -----------------------------------------------------------------------------------------

fn sample_scene(
    ray: &Ray,
    job: &mut Job,
    materials: &MaterialTable,
    bounces: &mut u32,
    max_bounces: u32,
    debug_normals: bool,
) -> Vec3 {
    let mut result = RayHitResult::MAX_HIT;

    // Manage bounces
    *bounces += 1;

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
    if debug_normals {
        return Vec3::new(
            (result.normal.x + 1.0) * 0.5,
            (result.normal.y + 1.0) * 0.5,
            (result.normal.z + 1.0) * 0.5,
        );
    }

    // Grab material
    let material = match materials.get(result.material) {
        Some(material) => material,
        None => {
            println!("Failed to find material: {}", result.material);
            &Material::NONE
        }
    };

    // Shade pixel (diffuse)
    let absorbed = 0.3;
    let reflected = 1.0 - absorbed;
    let refelcted_point = if result.material == "mirror" {
        result.position + Vec3::reflect(ray.direction, result.normal)
    } else {
        result.position + result.normal + (Vec3::random_point_in_unit_sphere(job.rng) * 0.99)
    };

    let reflected_ray_origin = result.position + (result.normal * 0.00001);
    let refelcted_ray_direction = Vec3::normalize(refelcted_point - result.position);
    let reflected_ray = Ray::new(reflected_ray_origin, refelcted_ray_direction);
    if *bounces < max_bounces {
        sample_scene(
            &reflected_ray,
            job,
            materials,
            bounces,
            max_bounces,
            debug_normals,
        ) * material.diffuse
            * reflected
    } else {
        material.diffuse * reflected
    }
}

// -----------------------------------------------------------------------------------------

fn sample_scene_spheres(ray: &Ray) -> RayHitResult {
    // Process all spheres
    let mut closest_result = RayHitResult::MAX_HIT;
    let mut closest_sphere_centre = Vec3::ZERO;
    for sphere in &SCENE_SPHERES {
        let result = raytrace::intersect::ray_sphere(&ray, &sphere);
        if result.hit && (result.distance < closest_result.distance) {
            closest_result = result;
            closest_sphere_centre = sphere.centre;
        }
    }

    // Calculate normal (if valid hit)
    if closest_result.hit {
        closest_result.normal = Vec3::normalize(closest_result.position - closest_sphere_centre);
        closest_result
    } else {
        RayHitResult::NO_HIT
    }
}

// -----------------------------------------------------------------------------------------

fn sample_scene_planes(ray: &Ray) -> RayHitResult {
    let plane = Plane {
        position: Vec3::ZERO,
        normal: Vec3::UP,
        diffuse: Vec3::ONE,
    };
    raytrace::intersect::ray_plane(&ray, &plane)
}

// -----------------------------------------------------------------------------------------
