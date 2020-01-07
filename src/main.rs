// -----------------------------------------------------------------------------------------
// Arrt dependencies
mod arrt;
use arrt::camera::Camera;
use arrt::camera::Tracer;
use arrt::command_line;
use arrt::geometry::Plane;
use arrt::geometry::Sphere;
use arrt::job::Job;
use arrt::material::MaterialBank;
use arrt::misc::StringLiteral;
use arrt::ray::Ray;
use arrt::ray::RayHitResult;
use arrt::vector::Vec3;

// -----------------------------------------------------------------------------------------
// External dependencies
use rand::prelude::*;
use winit::VirtualKeyCode;
use winit::MouseButton;

// -----------------------------------------------------------------------------------------
// Config
const PROGRESS_UPDATE_INTERVAL: f64 = 1.0;
const MATERIALS_FILE: StringLiteral = "materials.json";

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
const FLOOR_PLANE: Plane = Plane {
    position: Vec3::ZERO,
    normal: Vec3::UP,
    diffuse: Vec3::ONE,
};
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

fn main() {
    // Parse command line args
    let args = command_line::parse();

    // Load quality presets
    let quality_preset_name = args.value_of("quality").unwrap_or("default");
    let quality = arrt::quality::get_preset(quality_preset_name);

    // Load materials
    let materials = MaterialBank::load_from_file(MATERIALS_FILE);

    // Setup rng seed
    let rng_seed: u64 = args.occurrences_of("seed");

    // Setup camera
    let camera = Camera::new(CAMERA_POSITION, CAMERA_LOOKAT, CAMERA_FOV);

    // Setup job
    let debug_normals = args.is_present("debug-normals");
    let debug_heatmap = args.is_present("debug-heatmap");
    let mut job = Job::new(quality, materials, rng_seed, camera, debug_normals, debug_heatmap);

    // Run
    let is_interactive = args.is_present("interactive");
    if is_interactive
    {
        run_interactive(&mut job);
    }
    else
    {
        let output_filename = args.value_of("output-file").unwrap_or("output.bmp");
        run_headless(&mut job, output_filename);
    }
}

// -----------------------------------------------------------------------------------------

fn run_interactive(job : &mut Job)
{
    // Create window
    let mut window = mini_gl_fb::gotta_go_fast(
        "ARRT: Another Rust Ray Tracer",
        job.quality.image_width as f64,
        job.quality.image_height as f64,
    );

    // Pump message loop
    window.glutin_handle_basic_input(|window, input| {
        // Quit
        if input.key_is_down(VirtualKeyCode::Escape) {
            return false;
        }

        // Move forwards
        if input.mouse_is_down(MouseButton::Left) {
            job.camera.position.z += 1.0;
        }

        // Move backwards
        if input.mouse_is_down(MouseButton::Right) {
            job.camera.position.z -= 1.0;
        }

        // Redraw
        draw_scene(job);
        window.update_buffer(&job.image_buffer);
        true
    });
}

// -----------------------------------------------------------------------------------------

fn run_headless(job : &mut Job, output_filename : &str)
{
    // Start timer
    let timer_begin = time::precise_time_s();

    // Draw scene
    draw_scene(job);

    // Save image
    job.save_image(output_filename);

    // Stop timer and report
    let timer_end = time::precise_time_s();
    println!("");
    println!("====================================================");
    println!(" SUMMARY");
    println!("====================================================");
    println!("     Output: {}", output_filename);
    println!("       Seed: {}", job.rng_seed);
    println!("    Quality: {}", job.quality.name);
    println!(" Total time: {:.2} seconds", (timer_end - timer_begin));
    println!("====================================================");
}

// -----------------------------------------------------------------------------------------

fn sample_background(ray: &Ray) -> Vec3 {
    let t = (ray.direction.y + 1.0) * 0.5;
    Vec3::lerp(SKY_COLOUR_BOTTOM, SKY_COLOUR_TOP, t)
}

// -----------------------------------------------------------------------------------------

fn draw_scene(job: &mut Job) {
    // Setup camera
    let image_width = job.quality.image_width;
    let image_height = job.quality.image_height;
    let tracer = Tracer::new(&job.camera, image_width, image_height);

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
    let mut pixel = [0u8, 0u8, 0u8, 255u8];
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
            let mut bounces_per_pixel = 0;
            let mut bounces = 0;
            let ray_centroid = tracer.get_ray(pixel_x_f, pixel_y_f);
            let mut colour =
                sample_scene(&ray_centroid, job, &mut bounces, job.quality.max_bounces);
            bounces_per_pixel += bounces;

            // Take additional samples
            for sample_index in 0..additional_samples {
                let mut bounces = 0;
                let offset_x = (sample_offsets_x[sample_index] - 0.5) * 0.99;
                let offset_y = (sample_offsets_y[sample_index] - 0.5) * 0.99;
                let ray = tracer.get_ray(pixel_x_f + offset_x, pixel_y_f + offset_y);
                colour += sample_scene(&ray, job, &mut bounces, job.quality.max_bounces);
                bounces_per_pixel += bounces;
            }

            // Average colour
            colour /= job.quality.samples_per_pixel as f32;

            // Draw heatmap?
            if job.debug_heatmap {
                let heat = (bounces_per_pixel as f32 / job.quality.samples_per_pixel as f32) / job.quality.max_bounces as f32;
                colour = Vec3::lerp(Vec3::GREEN, Vec3::RED, heat);
            }

            // Store pixel
            Vec3::copy_to_pixel(colour, &mut pixel);

            // Write pixel
            let pixel_index = (pixel_y * image_width) + pixel_x;
            job.image_buffer[pixel_index as usize] = pixel;
        }
    }
}

// -----------------------------------------------------------------------------------------

fn sample_scene(ray: &Ray, job: &mut Job, bounces: &mut u32, max_bounces: u32) -> Vec3 {
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
    if job.debug_normals {
        return Vec3::new(
            (result.normal.x + 1.0) * 0.5,
            (result.normal.y + 1.0) * 0.5,
            (result.normal.z + 1.0) * 0.5,
        );
    }

    // Grab material
    let material = job.materials.get(result.material_name);

    // Shade pixel (diffuse)
    let reflected = 1.0 - material.absorbed;
    let refelcted_point = if material.name == "mirror" {
        result.position + Vec3::reflect(ray.direction, result.normal)
    } else {
        result.position + result.normal + (Vec3::random_point_in_unit_sphere(&mut job.rng) * 0.99)
    };

    let reflected_ray_origin = result.position + (result.normal * 0.00001);
    let refelcted_ray_direction = Vec3::normalize(refelcted_point - result.position);
    let reflected_ray = Ray::new(reflected_ray_origin, refelcted_ray_direction);
    if *bounces < max_bounces {
        let bounce_sample = sample_scene(&reflected_ray, job, bounces, max_bounces);
        bounce_sample * material.diffuse * reflected
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
        let result = arrt::intersect::ray_sphere(&ray, &sphere);
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
    arrt::intersect::ray_plane(&ray, &FLOOR_PLANE)
}

// -----------------------------------------------------------------------------------------
