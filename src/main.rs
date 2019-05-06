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

fn draw_scene(_image: &mut bmp::Image, _image_width: u32, _image_height: u32) {
    println!("Ready");
}

// -----------------------------------------------------------------------------------------
