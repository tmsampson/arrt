// -----------------------------------------------------------------------------------------
// Arrt dependencies
mod arrt;
use arrt::camera::Camera;
use arrt::command_line;
use arrt::geometry::Plane;
use arrt::geometry::Sphere;
use arrt::job::Job;
use arrt::material::MaterialBank;
use arrt::misc::StringLiteral;
use arrt::quality::QualityPresetBank;
use arrt::ray::Ray;
use arrt::ray::RayHitResult;
use arrt::vector::Vec3;

// -----------------------------------------------------------------------------------------
// External dependencies
use hotwatch::{Event, Hotwatch};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use winit::VirtualKeyCode;

// -----------------------------------------------------------------------------------------
// Config
const PROGRESS_UPDATE_INTERVAL: f64 = 1.0;
const QUALITY_PRESETS_FILE: StringLiteral = "quality_presets.json";
const MATERIALS_FILE: StringLiteral = "materials.json";
const EPSILON: f32 = 0.001;
const CAMERA_ROTATION_SPEED: f32 = 10.0;

// -----------------------------------------------------------------------------------------
// Config | Camera
const CAMERA_FOV: f32 = 90.0;
const CAMERA_POSITION: Vec3 = Vec3 {
    x: 0.0,
    y: 6.0,
    z: -20.0,
};
const CAMERA_LOOKAT: Vec3 = Vec3 {
    x: 0.0,
    y: 6.0,
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
    let quality_presets = QualityPresetBank::load_from_file(QUALITY_PRESETS_FILE);
    let quality_preset_name = args.value_of("quality").unwrap_or("default");
    let quality = quality_presets.get(quality_preset_name);

    // Load materials
    let materials = MaterialBank::load_from_file(MATERIALS_FILE);

    // Setup rng seed
    let rng_seed: u64 = args.occurrences_of("seed");

    // Setup camera
    let camera = Camera::new(CAMERA_POSITION, CAMERA_LOOKAT, CAMERA_FOV);

    // Setup job
    let debug_normals = args.is_present("debug-normals");
    let debug_heatmap = args.is_present("debug-heatmap");
    let mut job = Job::new(
        quality,
        materials,
        rng_seed,
        camera,
        debug_normals,
        debug_heatmap,
    );

    // Run
    let mode = args.value_of("mode").unwrap_or("interactive");
    match mode {
        "interactive" => run_interactive(&mut job),
        "headless" => {
            let output_filename = args.value_of("output-file").unwrap_or("output.bmp");
            run_headless(&mut job, output_filename);
        }
        _ => println!("Invalid mode"),
    }
}

// -----------------------------------------------------------------------------------------

fn run_interactive(job: &mut Job) {
    // Create window
    let mut window_handle = mini_gl_fb::gotta_go_fast(
        "ARRT: Another Rust Ray Tracer",
        job.quality.image_width as f64,
        job.quality.image_height as f64,
    );

    // Start listening for data file changes
    let mut watcher = Hotwatch::new().expect("File watcher failed to initialize!");
    let reload_materials_flag = Arc::new(AtomicBool::new(false));
    let reload_quality_flag = Arc::new(AtomicBool::new(false));
    watch_file(&mut watcher, MATERIALS_FILE, &reload_materials_flag);
    watch_file(&mut watcher, QUALITY_PRESETS_FILE, &reload_quality_flag);

    // Pump message loop
    let (mut draw_time_acc_s, mut present_time_acc_s) = (0.0, 0.0);
    let (mut frame_count, mut output_iteration) = (0, 0);
    let reload_materials_flag_consume = Arc::clone(&reload_materials_flag);
    let reload_quality_flag_consume = Arc::clone(&reload_quality_flag);
    window_handle.glutin_handle_basic_input(move |window, input| {
        // Live update materials?
        if reload_materials_flag_consume.load(Ordering::Relaxed) {
            reload_materials_flag_consume.swap(false, Ordering::Relaxed);
            println!("Reloading materials");
            job.materials = MaterialBank::load_from_file(MATERIALS_FILE);
        }

        // Live update quality?
        if reload_quality_flag_consume.load(Ordering::Relaxed) {
            reload_quality_flag_consume.swap(false, Ordering::Relaxed);
            println!("Reloading quality presets");
            let quality_presets = QualityPresetBank::load_from_file(QUALITY_PRESETS_FILE);
            job.quality = quality_presets.get(&job.quality.name);
        }

        // Quit
        if input.key_is_down(VirtualKeyCode::Escape) {
            return false;
        }

        // Apply camera movement
        const MOVEMENT_SPEED: f32 = 1.0;
        let mut update_camera = false;
        if input.key_is_down(VirtualKeyCode::W) {
            job.camera.position += job.camera.forward * MOVEMENT_SPEED; // Forwards
            update_camera = true;
        }
        if input.key_is_down(VirtualKeyCode::S) {
            job.camera.position -= job.camera.forward * MOVEMENT_SPEED; // Forwards
            update_camera = true;
        }
        if input.key_is_down(VirtualKeyCode::A) {
            job.camera.position -= job.camera.right * MOVEMENT_SPEED; // Left
            update_camera = true;
        }
        if input.key_is_down(VirtualKeyCode::D) {
            job.camera.position += job.camera.right * MOVEMENT_SPEED; // Right
            update_camera = true;
        }

        // Apply camera yaw
        let yaw_left = input.key_is_down(VirtualKeyCode::J);
        let yaw_right = input.key_is_down(VirtualKeyCode::L);
        if yaw_left || yaw_right {
            let angle = CAMERA_ROTATION_SPEED * if yaw_left { 1.0 } else { -1.0 };
            job.camera.forward = Vec3::rotate_yaxis(job.camera.forward, angle);
            job.camera.right = Vec3::cross(Vec3::UP, job.camera.forward);
            job.camera.up = Vec3::cross(job.camera.forward, job.camera.right);
            update_camera = true;
        }

        // Apply camera pitch
        let pitch_up = input.key_is_down(VirtualKeyCode::I);
        let pitch_down = input.key_is_down(VirtualKeyCode::K);
        if pitch_up || pitch_down {
            let angle = CAMERA_ROTATION_SPEED * if pitch_up { 1.0 } else { -1.0 };
            job.camera.forward = Vec3::rotate(job.camera.forward, job.camera.right, angle);
            job.camera.up = Vec3::cross(job.camera.forward, job.camera.right);
            update_camera = true;
        }

        // Update camera
        if(update_camera)
        {
            job.camera.update_cached_rays(job.quality.image_width, job.quality.image_height, job.quality.samples_per_pixel, &mut job.rng);
        }

        // Redraw
        let timer_draw_begin = time::precise_time_s();
        draw_scene(job);
        let timer_draw_end = time::precise_time_s();

        // Present
        let timer_present_begin = time::precise_time_s();
        window.update_buffer(&job.image_buffer);
        let timer_present_end = time::precise_time_s();

        // Update timer
        let draw_time_s = timer_draw_end - timer_draw_begin;
        let present_time_s = timer_present_end - timer_present_begin;
        draw_time_acc_s += draw_time_s;
        present_time_acc_s += present_time_s;
        frame_count += 1;

        // Show profiling info
        let total_time = draw_time_acc_s + present_time_acc_s;
        if total_time > 1.0 {
            let average_ms = (total_time / frame_count as f64) * 1000.0;
            let average_draw_ms = (draw_time_acc_s / frame_count as f64) * 1000.0;
            let average_present_ms = (present_time_acc_s / frame_count as f64) * 1000.0;
            let average_fps = 1000.0 / average_ms;
            println!(
                "{}|{:.2}|{:.2}|{:.2}|{:.2}",
                output_iteration, average_fps, average_ms, average_draw_ms, average_present_ms
            );
            draw_time_acc_s = 0.0;
            present_time_acc_s = 0.0;
            frame_count = 0;
            output_iteration += 1;
        }
        true
    });
}

// -----------------------------------------------------------------------------------------

fn run_headless(job: &mut Job, output_filename: &str) {
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

fn draw_scene(job: &mut Job) {
    // Setup state
    let image_width = job.quality.image_width;
    let image_height = job.quality.image_height;

    // Setup regular progress updates
    //let mut last_progress_update = time::precise_time_s();

    // For each scanline...
    let mut pixel = [0u8, 0u8, 0u8, 255u8];
    for pixel_y in 0..image_height {
        // Show progress?
        // let now = time::precise_time_s();
        // let elapsed = now - last_progress_update;
        // if elapsed >= PROGRESS_UPDATE_INTERVAL {
        //     let row = pixel_y + 1;
        //     let percent = ((row as f32) / (image_height as f32)) * 100.0;
        //     println!(
        //         "Tracing: {:.2}% complete scanline {} / {}",
        //         percent, row, image_height
        //     );
        //     last_progress_update = now;
        // }

        // For each column
        for pixel_x in 0..image_width {
            let pixel_index = Camera::get_pixel_index(
                pixel_x,
                pixel_y,
                image_width,
                job.quality.samples_per_pixel,
            );

            let mut colour = Vec3::BLACK;
            let mut bounces_per_pixel = 0;
            for sample_index in 0..job.quality.samples_per_pixel {
                let mut bounces = 0;
                let ray = job.camera.cached_rays[pixel_index + sample_index];
                colour += sample_scene(&ray, job, &mut bounces);
                bounces_per_pixel += bounces;
            }

            // Average colour
            colour /= job.quality.samples_per_pixel as f32;

            // Draw heatmap?
            if job.debug_heatmap {
                let heat = (bounces_per_pixel as f32 / job.quality.samples_per_pixel as f32)
                    / job.quality.max_bounces as f32;
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

fn sample_scene(ray: &Ray, job: &mut Job, bounces: &mut u32) -> Vec3 {
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

    // Test against sdf
    // let sdf_result = sample_scene_sdf(ray);
    // if sdf_result.hit && sdf_result.distance < result.distance {
    //     result = sdf_result;
    // }

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

    let reflected_ray_origin = result.position + (result.normal * EPSILON);
    let refelcted_ray_direction = Vec3::normalize(refelcted_point - result.position);
    let reflected_ray = Ray::new(reflected_ray_origin, refelcted_ray_direction);
    if *bounces <= job.quality.max_bounces {
        let bounce_sample = sample_scene(&reflected_ray, job, bounces);
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

fn sample_scene_sdf(ray: &Ray) -> RayHitResult {
    let sphere_origin = Vec3::new(0.0, 9.5, 0.0);
    let sphere_radius = 3.0;
    let sphere_material = "green";

    let mut ray_pos = ray.origin;
    const MAX_STEPS: u32 = 50;
    for _x in 0..MAX_STEPS {
        let sphere_dist = sdf_sphere(sphere_origin, sphere_radius, ray_pos);
        if sphere_dist < EPSILON {
            let normal = Vec3::normalize(ray_pos - sphere_origin);
            let dist = Vec3::length(ray_pos - ray.origin);
            return RayHitResult::new(true, dist, ray_pos, normal, sphere_material);
        }
        ray_pos += ray.direction * sphere_dist;
    }
    RayHitResult::NO_HIT
}

// -----------------------------------------------------------------------------------------

fn sdf_sphere(origin: Vec3, radius: f32, sample: Vec3) -> f32 {
    // https://iquilezles.org/www/articles/distfunctions/distfunctions.htm
    let to_origin = origin - sample;
    Vec3::length(to_origin) - radius
}

// -----------------------------------------------------------------------------------------

fn sample_background(ray: &Ray) -> Vec3 {
    let t = (ray.direction.y + 1.0) * 0.5;
    Vec3::lerp(SKY_COLOUR_BOTTOM, SKY_COLOUR_TOP, t)
}

// -----------------------------------------------------------------------------------------

fn watch_file(watcher: &mut hotwatch::Hotwatch, file: &str, flag: &Arc<AtomicBool>) {
    let flag_shared = Arc::clone(&flag);
    watcher
        .watch(file, move |event: Event| {
            if let Event::Write(_path) = event {
                flag_shared.swap(true, Ordering::Relaxed);
            }
        })
        .expect("Failed to watch file!");
}

// -----------------------------------------------------------------------------------------
