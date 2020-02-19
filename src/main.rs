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
use num_cpus;
use rand::prelude::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use winit::VirtualKeyCode;

// -----------------------------------------------------------------------------------------
// Config
const _PROGRESS_UPDATE_INTERVAL: f64 = 1.0;
const QUALITY_PRESETS_FILE: StringLiteral = "quality_presets.json";
const MATERIALS_FILE: StringLiteral = "materials.json";
const EPSILON: f32 = 0.001;
const CAMERA_ROTATION_SPEED: f32 = 2.0;

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
    // // Parse command line args
    // let args = command_line::parse();

    // // Load quality presets
    // let quality_presets = QualityPresetBank::load_from_file(QUALITY_PRESETS_FILE);
    // let quality_preset_name = args.value_of("quality").unwrap_or("default");
    // let quality = quality_presets.get(quality_preset_name);

    // // Load materials
    // let materials = MaterialBank::load_from_file(MATERIALS_FILE);

    // // Setup rng seed
    // let rng_seed: u64 = args.occurrences_of("seed");

    // // Setup camera
    // let camera = Camera::new(CAMERA_POSITION, CAMERA_LOOKAT, CAMERA_FOV);

    // // Setup job
    // let debug_normals = args.is_present("debug-normals");
    // let debug_heatmap = args.is_present("debug-heatmap");
    // let mut job = Job::new(
    //     quality,
    //     materials,
    //     rng_seed,
    //     camera,
    //     debug_normals,
    //     debug_heatmap,
    // );

    // Run
    run_interactive();
    // let mode = args.value_of("mode").unwrap_or("interactive");
    // match mode {
    //     "interactive" => run_interactive(),
    //     "headless" => {
    //         let output_filename = args.value_of("output-file").unwrap_or("output.bmp");
    //         run_headless(&mut job, output_filename);
    //     }
    //     _ => println!("Invalid mode"),
    // }
}

// -----------------------------------------------------------------------------------------

//type SharedJob = std::sync::Arc<arrt::job::Job>;
type SafeQueue = std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<RayJob>>>;

#[derive(Debug, Default, Copy, Clone)]
pub struct RayJob {
    pub pixel_index: usize,
    pub sample_index: usize,
    pub ray_index: usize,
    pub bounce_index: u32,
    pub ray: Ray,
    pub max_bounces: u32,
    pub movement_counter: u64,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RayJobResult {
    pub pixel_index: usize,
    pub sample_index: usize,
    pub ray_index: usize,
    pub bounce_index: u32,
    pub colour: Vec3,
}

// -----------------------------------------------------------------------------------------

fn run_thread(
    thread_index: usize,
    rng_seed: u64,
    job_queue_arc: &SafeQueue,
    job_arc: &Job,
    tx: Transmitter,
) {
    let mut rng = SeedableRng::seed_from_u64(rng_seed);

    loop {
        let mut job_queue = job_queue_arc.lock().unwrap();
        if job_queue.len() > 0 {
            let ray_job = job_queue.pop_front().unwrap();
            drop(job_queue); // release lock
            let ray = &ray_job.ray;
            let (colour, result) = sample_scene(ray, &job_arc);

            // Transmit result
            let job_result = RayJobResult {
                pixel_index: ray_job.pixel_index,
                sample_index: ray_job.sample_index,
                ray_index: ray_job.ray_index,
                bounce_index: ray_job.bounce_index,
                colour,
            };
            // println!(
            //     "[THREAD] send: pixel_index = {} ray_index = {}, colour = {}, {}, {}",
            //     job_result.pixel_index,
            //     job_result.ray_index,
            //     job_result.colour.x,
            //     job_result.colour.y,
            //     job_result.colour.z,
            // );
            tx.send(job_result).unwrap();

            // Schedule bounce job?
            if result.hit && ((ray_job.bounce_index + 1) <= ray_job.max_bounces) {
                // Calculate reflected point
                let material = job_arc.materials.get(result.material_name);
                let refelcted_point = if material.name == "mirror" {
                    result.position + Vec3::reflect(ray.direction, result.normal)
                } else {
                    result.position
                        + result.normal
                        + (Vec3::random_point_in_unit_sphere(&mut rng) * 0.99)
                };

                // Calculate reflected ray
                let reflected_ray_origin = result.position + (result.normal * EPSILON);
                let refelcted_ray_direction = Vec3::normalize(refelcted_point - result.position);
                let reflected_ray = Ray::new(reflected_ray_origin, refelcted_ray_direction);

                // Setup bounce job
                let mut bounce_job = ray_job;
                bounce_job.ray = reflected_ray;
                bounce_job.bounce_index = ray_job.bounce_index + 1;

                // Lock queue and schedule bounce job
                let mut job_queue = job_queue_arc.lock().unwrap();
                job_queue.push_back(bounce_job);
            }
        } else {
            // TODO: wait on event here which is raised when camera moves!
            // (*job_queue).push_back(2);// schedule next bounce
            drop(job_queue); // release lock
                             //println!("Thread {} starved", thread_index);
            thread::sleep(Duration::from_millis(100));
        }
    }
}

// -----------------------------------------------------------------------------------------
type JobQueue = VecDeque<RayJob>;
type Transmitter = std::sync::mpsc::Sender<RayJobResult>;

fn schedule_work(
    image_width: u32,
    image_height: u32,
    samples_per_pixel: usize,
    max_bounces: u32,
    camera: &Camera,
    job_queue: &mut JobQueue,
    movement_counter: u64,
) {
    job_queue.clear();
    for sample_index in 0..samples_per_pixel {
        for pixel_y in 0..image_height {
            for pixel_x in 0..image_width {
                let pixel_index = (((pixel_y * image_width) + pixel_x) as usize) * samples_per_pixel;
                    let ray_index = pixel_index + sample_index;
                    let ray = camera.cached_rays[ray_index as usize];
                    let ray_job = RayJob {
                        pixel_index,
                        sample_index,
                        ray_index,
                        ray,
                        bounce_index: 0,
                        movement_counter,
                        max_bounces,
                    };
                    job_queue.push_back(ray_job);
            }
        }
    }
}

// -----------------------------------------------------------------------------------------

fn run_interactive() {
    // Parse command line args
    let args = command_line::parse();

    // Load quality presets
    let quality_presets = QualityPresetBank::load_from_file(QUALITY_PRESETS_FILE);
    let quality_preset_name = args.value_of("quality").unwrap_or("default");
    let quality = quality_presets.get(quality_preset_name);

    // Load materials
    let materials = MaterialBank::load_from_file(MATERIALS_FILE);

    // Setup rng
    let rng_seed: u64 = args.occurrences_of("seed");
    let mut rng = SeedableRng::seed_from_u64(rng_seed);

    // Setup camera
    let mut camera = Camera::new(CAMERA_POSITION, CAMERA_LOOKAT, CAMERA_FOV);

    // Cache camera rays
    camera.update_cached_rays(
        quality.image_width,
        quality.image_height,
        quality.samples_per_pixel,
        &mut rng,
    );

    // Setup job
    let debug_normals = args.is_present("debug-normals");
    let debug_heatmap = args.is_present("debug-heatmap");
    let (image_width, image_height) = (quality.image_width, quality.image_height);
    let samples_per_pixel = quality.samples_per_pixel;
    let max_bounces = quality.max_bounces;
    let job = Job::new(quality, materials, debug_normals, debug_heatmap);

    // Setup image buffer
    let total_pixel_count = image_height * image_width;
    let clear_colour = [0u8, 0u8, 0u8, 255u8];
    let mut image_buffer = vec![clear_colour; total_pixel_count as usize];

    // Create window
    let mut window_handle = mini_gl_fb::gotta_go_fast(
        "ARRT: Another Rust Ray Tracer",
        image_width as f64,
        image_height as f64,
    );

    // Start listening for data file changes
    let mut watcher = Hotwatch::new().expect("File watcher failed to initialize!");
    let reload_materials_flag = Arc::new(AtomicBool::new(false));
    let reload_quality_flag = Arc::new(AtomicBool::new(false));
    watch_file(&mut watcher, MATERIALS_FILE, &reload_materials_flag);
    watch_file(&mut watcher, QUALITY_PRESETS_FILE, &reload_quality_flag);

    // Setup result store
    let total_ray_job_count = (total_pixel_count as usize) * samples_per_pixel;
    let mut result_store = vec![Vec3::BLACK; total_ray_job_count];

    // Setup result queue
    let (tx, rx) = mpsc::channel::<RayJobResult>();

    // Schedule work
    let mut movement_counter = 0;
    let total_ray_job_count = (total_pixel_count as usize) * samples_per_pixel;
    let mut job_queue = JobQueue::with_capacity(total_ray_job_count as usize);
    schedule_work(
        image_width,
        image_height,
        samples_per_pixel,
        max_bounces,
        &camera,
        &mut job_queue,
        movement_counter
    );

    // Threading
    let job_arc = Arc::new(job);
    let job_queue_arc = Arc::new(Mutex::new(job_queue));
    let thread_count = num_cpus::get();
    for thread_index in 0..thread_count {
        let job_arc = job_arc.clone();
        let job_queue_arc = job_queue_arc.clone();
        let tx = mpsc::Sender::clone(&tx);
        thread::spawn(move || {
            run_thread(thread_index, rng_seed, &job_queue_arc, &job_arc, tx);
        });
    }

    // Pump message loop
    let job = job_arc.clone();
    let (mut draw_time_acc_s, mut present_time_acc_s) = (0.0, 0.0);
    let (mut frame_count, mut total_frame_count, mut output_iteration) = (0, 0, 0);
    let _reload_materials_flag_consume = Arc::clone(&reload_materials_flag);
    let _reload_quality_flag_consume = Arc::clone(&reload_quality_flag);
    window_handle.glutin_handle_basic_input(move |window, input| {
        // Live update materials?
        // if reload_materials_flag_consume.load(Ordering::Relaxed) {
        //     reload_materials_flag_consume.swap(false, Ordering::Relaxed);
        //     println!("Reloading materials");
        //     job.materials = MaterialBank::load_from_file(MATERIALS_FILE);
        // }

        // // Live update quality?
        // if reload_quality_flag_consume.load(Ordering::Relaxed) {
        //     reload_quality_flag_consume.swap(false, Ordering::Relaxed);
        //     println!("Reloading quality presets");
        //     let quality_presets = QualityPresetBank::load_from_file(QUALITY_PRESETS_FILE);
        //     job.quality = quality_presets.get(&job.quality.name);
        // }

        // Quit
        if input.key_is_down(VirtualKeyCode::Escape) {
            return false;
        }

         // Apply camera movement
         const MOVEMENT_SPEED: f32 = 0.2;
         let mut update_camera = false;
         if input.key_is_down(VirtualKeyCode::W) {
             camera.position += camera.forward * MOVEMENT_SPEED; // Forwards
             update_camera = true;
         }
         if input.key_is_down(VirtualKeyCode::S) {
             camera.position -= camera.forward * MOVEMENT_SPEED; // Forwards
             update_camera = true;
         }
         if input.key_is_down(VirtualKeyCode::A) {
             camera.position -= camera.right * MOVEMENT_SPEED; // Left
             update_camera = true;
         }
         if input.key_is_down(VirtualKeyCode::D) {
             camera.position += camera.right * MOVEMENT_SPEED; // Right
             update_camera = true;
         }
 
         // Apply camera yaw
         let yaw_left = input.key_is_down(VirtualKeyCode::J);
         let yaw_right = input.key_is_down(VirtualKeyCode::L);
         if yaw_left || yaw_right {
             let angle = CAMERA_ROTATION_SPEED * if yaw_left { 1.0 } else { -1.0 };
             camera.forward = Vec3::rotate_yaxis(camera.forward, angle);
             camera.right = Vec3::cross(Vec3::UP, camera.forward);
             camera.up = Vec3::cross(camera.forward, camera.right);
             update_camera = true;
         }
 
         // Apply camera pitch
         let pitch_up = input.key_is_down(VirtualKeyCode::I);
         let pitch_down = input.key_is_down(VirtualKeyCode::K);
         if pitch_up || pitch_down {
             let angle = CAMERA_ROTATION_SPEED * if pitch_up { 1.0 } else { -1.0 };
             camera.forward = Vec3::rotate(camera.forward, camera.right, angle);
             camera.up = Vec3::cross(camera.forward, camera.right);
             update_camera = true;
         }
 
         // Update camera
         if update_camera {
             camera.update_cached_rays(
                 job.quality.image_width,
                 job.quality.image_height,
                 job.quality.samples_per_pixel,
                 &mut rng,
             );

             // Clear results
             for i in 0..result_store.len()
             {
                result_store[i] = Vec3::BLACK;
             }
 
             // Lock and re-schedule work
             movement_counter = movement_counter + 1;
             let mut job_queue = job_queue_arc.lock().unwrap();
             schedule_work(
                 image_width,
                 image_height,
                 samples_per_pixel,
                 max_bounces,
                 &camera,
                 &mut job_queue,
                 movement_counter,
             );
             return true;
         }

        // Receive results from threads
        let iter = rx.try_iter();
        for result in iter.take((image_width * image_height) as usize) {
            // println!(
            //     "[MAIN] rcv: pixel_index = {} ray_index = {}, colour = {}, {}, {}",
            //     result.pixel_index,
            //     result.ray_index,
            //     result.colour.x,
            //     result.colour.y,
            //     result.colour.z
            // );
            let first_result = result.bounce_index == 0;
            if first_result {
                result_store[result.ray_index] = result.colour;
            } else {
                result_store[result.ray_index] *= result.colour;
            }
        }
        // Redraw
        let timer_draw_begin = time::precise_time_s();
        let mut pixel = [0u8, 0u8, 0u8, 255u8];
        for pixel_y in 0..image_height {
            for pixel_x in 0..image_width {
                let mut average = Vec3::BLACK;
                let pixel_index =
                    (((pixel_y * image_width) + pixel_x) as usize) * samples_per_pixel;
                for sample_index in 0..samples_per_pixel {
                    let ray_index = pixel_index + sample_index;
                    average += result_store[ray_index];
                }
                average /= samples_per_pixel as f32;
                // println!(
                //     "[MAIN] write: colour = {}, {}, {}",
                //     average.x, average.y, average.z
                // );
                let pixel_index_2 = (((pixel_y * image_width) + pixel_x) as usize);
                Vec3::copy_to_pixel(average, &mut pixel);

                // Write pixel
                image_buffer[pixel_index_2 as usize] = pixel;
            }
        }
        // draw_scene(job, false);
        let timer_draw_end = time::precise_time_s();

        // Present
        let timer_present_begin = time::precise_time_s();
        window.update_buffer(&image_buffer);
        let timer_present_end = time::precise_time_s();

        // Update timer
        let draw_time_s = timer_draw_end - timer_draw_begin;
        let present_time_s = timer_present_end - timer_present_begin;
        draw_time_acc_s += draw_time_s;
        present_time_acc_s += present_time_s;
        frame_count += 1;
        total_frame_count += 1;

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

fn _run_headless(_job: &mut Job, _output_filename: &str) {
    // Start timer
    // let timer_begin = time::precise_time_s();

    // // Draw scene
    // draw_scene(job, true);

    // // Save image
    // job.save_image(output_filename);

    // // Stop timer and report
    // let timer_end = time::precise_time_s();
    // println!("");
    // println!("====================================================");
    // println!(" SUMMARY");
    // println!("====================================================");
    // println!("     Output: {}", output_filename);
    // //println!("       Seed: {}", rng_seed);
    // println!("    Quality: {}", job.quality.name);
    // println!(" Total time: {:.2} seconds", (timer_end - timer_begin));
    // println!("====================================================");
}

// -----------------------------------------------------------------------------------------

fn _draw_scene(job: &mut Job, output_progress_updates: bool) {
    // Setup state
    let image_width = job.quality.image_width;
    let image_height = job.quality.image_height;

    // Setup timer?
    // let mut last_progress_update = if output_progress_updates {
    //     time::precise_time_s()
    // } else {
    //     0.0
    // };

    // For each scanline...
    let mut pixel = [0u8, 0u8, 0u8, 255u8];
    for pixel_y in 0..image_height {
        // Show progress?
        // if output_progress_updates {
        //     let now = time::precise_time_s();
        //     let elapsed = now - last_progress_update;
        //     if elapsed >= PROGRESS_UPDATE_INTERVAL {
        //         let row = pixel_y + 1;
        //         let percent = ((row as f32) / (image_height as f32)) * 100.0;
        //         println!(
        //             "Tracing: {:.2}% complete scanline {} / {}",
        //             percent, row, image_height
        //         );
        //         last_progress_update = now;
        //     }
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
                //let ray = camera.cached_rays[pixel_index + sample_index];
                //colour += sample_scene(&ray, job, &mut bounces);
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
            //job.image_buffer[pixel_index as usize] = pixel;
        }
    }
}

// -----------------------------------------------------------------------------------------

fn sample_scene(ray: &Ray, job: &Job) -> (Vec3, RayHitResult) {
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

    // Test against sdf
    // let sdf_result = _sample_scene_sdf(ray);
    // if sdf_result.hit && sdf_result.distance < result.distance {
    //     result = sdf_result;
    // }

    // Use background?
    let mut colour: Vec3;
    if job.debug_normals {
        colour = Vec3::new(
            (result.normal.x + 1.0) * 0.5,
            (result.normal.y + 1.0) * 0.5,
            (result.normal.z + 1.0) * 0.5,
        );
    } else {
        if !result.hit {
            colour = sample_background(ray);
        } else {
            // Grab material
            let material = job.materials.get(result.material_name);

            // Shade pixel (diffuse)
            let reflected = 1.0 - material.absorbed;
            colour = material.diffuse * reflected;
        }
    }

    // Return info
    (colour, result)
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

fn _sample_scene_sdf(ray: &Ray) -> RayHitResult {
    let sphere_origin = Vec3::new(0.0, 9.5, 0.0);
    let sphere_radius = 3.0;
    let sphere_material = "green";

    let mut ray_pos = ray.origin;
    const MAX_STEPS: u32 = 50;
    for _x in 0..MAX_STEPS {
        let sphere_dist = _sdf_sphere(sphere_origin, sphere_radius, ray_pos);
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

fn _sdf_sphere(origin: Vec3, radius: f32, sample: Vec3) -> f32 {
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
