//#[feature(edition2021)]
#![feature(portable_simd)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use doomgeneric::doom::{self, KeyData};
use libremarkable::cgmath::Point2;
use libremarkable::framebuffer::common;
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{
    FramebufferBase, FramebufferDraw, FramebufferIO, FramebufferRefresh,
};
use libremarkable::image::RgbImage;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

mod blue_noise_dither;

struct Game {
    image: std::sync::Arc<std::sync::Mutex<RgbImage>>,
    input_queue: VecDeque<KeyData>,
}

/*
fn button_to_doom_key(button: Button) -> Option<u8> {
        match button {
        Button::Keyboard(key) => match key {
            // Map keyboard keys from m_controller.c
            Key::Right => Some(unsafe { doom::key_right as u8 }),
            Key::Left => Some(unsafe { doom::key_left as u8 }),
            Key::Up => Some(unsafe { doom::key_up as u8 }),
            Key::Down => Some(unsafe { doom::key_down as u8 }),
            Key::Comma => Some(unsafe { doom::key_strafeleft as u8 }),
            Key::Period => Some(unsafe { doom::key_straferight as u8 }),
            Key::RCtrl => Some(unsafe { doom::key_fire as u8 }),
            Key::Space => Some(unsafe { doom::key_use as u8 }),
            Key::LAlt | Key::RAlt => Some(unsafe { doom::key_strafe as u8 }),
            Key::LShift | Key::RShift => Some(unsafe { doom::key_speed as u8 }),
            // Let doom deal with the rest
            _ => Some(key as u8),
        },
        _ => None,
    }
}*/

impl doom::Doom for Game {
    fn draw_frame(&mut self, screen_buffer: &[u32], xres: usize, yres: usize) {
        let mut rgb_img = libremarkable::image::RgbImage::new(xres as u32, yres as u32);
        assert!(xres * yres == screen_buffer.len());

        for (index, argb) in screen_buffer.iter().enumerate() {
            let pixel = libremarkable::image::Rgb([
                ((argb >> 16) & 0xFF) as u8,
                ((argb >> 8) & 0xFF) as u8,
                ((argb >> 0) & 0xFF) as u8,
            ]);
            let x = (index % xres) as u32;
            let y = (index / xres) as u32;

            rgb_img.put_pixel(x, y, pixel);
        }

        *self.image.lock().unwrap() = rgb_img;
    }
    fn get_key(&mut self) -> Option<doom::KeyData> {
        self.input_queue.pop_front()
    }
    fn set_window_title(&mut self, _title: &str) {
        //self.indow.ctx.window().set_title(title);
    }
}

/*extern "C" {
    // arm_neon.h from toolchain
    fn vcgt_u8 (uint8x8_t __a, uint8x8_t __b);
}*/

fn main() {
    let mut fb = Framebuffer::from_path("/dev/fb0");
    fb.clear();
    fb.full_refresh(
        common::waveform_mode::WAVEFORM_MODE_INIT,
        common::display_temp::TEMP_USE_MAX,
        common::dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
        0,
        true,
    );
    let start = Instant::now();
    let mut ditherer = blue_noise_dither::CachedDither2XTo4X::new(
        doom::DOOMGENERIC_RESX as u32,
        doom::DOOMGENERIC_RESY as u32,
    );
    println!("Precalculated dither in {:?}", start.elapsed());

    let image = std::sync::Arc::new(std::sync::Mutex::new(RgbImage::new(
        doom::DOOMGENERIC_RESX as u32,
        doom::DOOMGENERIC_RESY as u32,
    )));
    let image_clone = image.clone();
    std::thread::spawn(move || {
        let mut last_frame_drawn = Instant::now() - Duration::from_millis(1000);
        let width = doom::DOOMGENERIC_RESX as u32 * SCALE_FACTOR as u32;
        let height = doom::DOOMGENERIC_RESY as u32 * SCALE_FACTOR as u32;
        let pos = Point2 {
            x: (common::DISPLAYWIDTH as i32 - width as i32) / 2,
            y: (common::DISPLAYHEIGHT as i32 - height as i32) / 2,
        };

        loop {
            // Useing this to artificially lower the refresh rate on rm2
            if libremarkable::device::CURRENT_DEVICE.model == libremarkable::device::Model::Gen2
                && last_frame_drawn.elapsed() < Duration::from_millis(750)
            {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            // Useing this to artificially lower the refresh rate on rm2
            if libremarkable::device::CURRENT_DEVICE.model == libremarkable::device::Model::Gen1
                && last_frame_drawn.elapsed() < Duration::from_millis(20)
            {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }

            let rgb_img = &image.lock().unwrap().clone();
            let start = Instant::now();
            //let dithered_img = blue_noise_dither::dither_image(rgb_img, SCALE_FACTOR);
            let dithered_img = ditherer.dither_image(rgb_img);
            println!("Waited {:?} to dither image!", start.elapsed());

            let start = Instant::now();
            //fb.draw_image(&dithered_img, pos);
            draw_image_mono(&mut fb, pos, &dithered_img);
            fb.partial_refresh(
                &common::mxcfb_rect {
                    left: pos.x as u32,
                    top: pos.y as u32,
                    width,
                    height,
                },
                libremarkable::framebuffer::refresh::PartialRefreshMode::Async,
                common::waveform_mode::WAVEFORM_MODE_DU,
                common::display_temp::TEMP_USE_REMARKABLE_DRAW,
                common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
                0,
                false,
            );
            println!("Waited {:?} to draw image!", start.elapsed());
            last_frame_drawn = Instant::now();
        }
    });

    doom::init(Game {
        image: image_clone,
        input_queue: VecDeque::new(),
    });
}

fn draw_image_mono(fb: &mut Framebuffer, pos: Point2<i32>, img: &libremarkable::image::GrayImage) {
    /*for (x, y, pixel) in img.enumerate_pixels() {
        let pixel_pos = pos + vec2(x as i32, y as i32);
        fb.write_pixel(
            pixel_pos.cast().unwrap(),
            if pixel.data[0] > 0 {
                common::color::WHITE
            } else {
                common::color::BLACK
            },
        );
    }*/

    let width = img.width();
    let height = img.height();
    let mut fb_raw_data: Vec<u8> =
        Vec::with_capacity(img.width() as usize * 2 * img.height() as usize);
    let img_vec = img.to_vec();
    //let x_abs_end = pos.x + img.width();
    //let y_abs_end = pos.y + img.height();
    //let start = Instant::now();
    for pixel_value in img_vec {
        fb_raw_data.push(pixel_value);
        fb_raw_data.push(pixel_value);
    }
    //println!("Draw: Loop took {:?}", start.elapsed());

    /*for img_pixel_index in 0..img_vec.len() {
        let pixel_value = img_vec[img_pixel_index];
        fb_raw_data.push(pixel_value);
        fb_raw_data.push(pixel_value);
    }*/
    fb.restore_region(
        common::mxcfb_rect {
            top: pos.y as u32,
            left: pos.x as u32,
            width,
            height,
        },
        &fb_raw_data,
    )
    .unwrap();

    /*
    for (x, y, pixel) in img.enumerate_pixels() {
        let pixel_pos = pos + vec2(x as i32, y as i32);
        fb.write_pixel(
            pixel_pos.cast().unwrap(),
            if pixel.data[0] > 0 {
                common::color::WHITE
            } else {
                common::color::BLACK
            },
        );
    }*/

    /*
    let img_vec = img.to_vec();
    for y in 0..img.height() as i32 {
        let y_final = pos.y + y;
        for x_from in 0..(img.width() as usize / 32) {
            let x_to_excl = x_from + 32;
            let mut pixels_chunk: [u8; 32] = [0u8; 32];
            pixels_chunk.copy_from_slice(&img_vec[x_from..x_to_excl]);
            //img_vec[x_from..x_to_excl]
            let pixels = u8x32::from_array(pixels_chunk);
            pixels.mul(255);
        }
    }*/
}
