use ab_glyph::{FontRef, PxScale, Font, ScaleFont};
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;

const ICON_SIZE: u32 = 44;
const FONT_SIZE: f32 = 36.0;
const FONT_SIZE_100: f32 = 30.0; // Smaller font for 100%

pub fn generate_battery_icon(percentage: u8) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Determine if we should use template mode or color mode
    let use_template = percentage > 50;

    // Load embedded bold font
    let font_data = include_bytes!("../assets/font-bold.ttf");
    let font = FontRef::try_from_slice(font_data)?;

    // Prepare text
    let text = format!("{}", percentage);
    // Use smaller font size for 100%
    let font_size = if percentage == 100 { FONT_SIZE_100 } else { FONT_SIZE };
    let scale = PxScale::from(font_size);

    // Calculate text width for proper centering
    let scaled_font = font.as_scaled(scale);
    let mut text_width = 0.0;
    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        text_width += scaled_font.h_advance(glyph_id);
    }

    let img: RgbaImage = if use_template {
        // Template mode: Create cutout effect (transparent text on solid background)
        // Start with solid black background
        let mut img: RgbaImage = ImageBuffer::from_pixel(ICON_SIZE, ICON_SIZE, Rgba([0u8, 0, 0, 255]));

        // Create a temporary image with white text on transparent background
        let mut text_img: RgbaImage = ImageBuffer::from_pixel(ICON_SIZE, ICON_SIZE, Rgba([0u8, 0, 0, 0]));
        let start_x = ((ICON_SIZE as f32 - text_width) / 2.0).max(0.0) as i32;
        let start_y = if percentage == 100 { 6i32 } else { 3i32 };

        draw_text_mut(
            &mut text_img,
            Rgba([255, 255, 255, 255]),
            start_x,
            start_y,
            scale,
            &font,
            &text,
        );

        // Cut out the text from the black background (make text transparent)
        for y in 0..ICON_SIZE {
            for x in 0..ICON_SIZE {
                let text_pixel = *text_img.get_pixel(x, y);
                if text_pixel[3] > 0 {
                    // Where text exists, make it transparent
                    *img.get_pixel_mut(x, y) = Rgba([0, 0, 0, 0]);
                }
            }
        }

        img
    } else {
        // Color mode: Solid background with white text
        let bg_color = if percentage <= 20 {
            Rgba([255u8, 0, 0, 255]) // Bright Red
        } else {
            Rgba([255u8, 120, 0, 255]) // Darker Orange
        };

        let mut img: RgbaImage = ImageBuffer::from_pixel(ICON_SIZE, ICON_SIZE, bg_color);
        let start_x = ((ICON_SIZE as f32 - text_width) / 2.0).max(0.0) as i32;
        let start_y = if percentage == 100 { 6i32 } else { 3i32 };

        draw_text_mut(
            &mut img,
            Rgba([255, 255, 255, 255]),
            start_x,
            start_y,
            scale,
            &font,
            &text,
        );

        img
    };

    // Convert to PNG bytes
    let mut png_bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    )?;

    Ok(png_bytes)
}
