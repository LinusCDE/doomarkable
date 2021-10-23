//#[feature(edition2021)]
//#![feature(portable_simd)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use doomgeneric::doom::{self, KeyData};
use libremarkable::cgmath::{Point2, Vector2};
use libremarkable::framebuffer::common;
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{
    FramebufferBase, FramebufferDraw, FramebufferIO, FramebufferRefresh,
};
use libremarkable::image::RgbImage;
use libremarkable::input::{
    ev::EvDevContext, multitouch::Finger, multitouch::MultitouchEvent, InputDevice, InputEvent,
};
use std::time::{Duration, Instant};

mod blue_noise_dither;

const SCALE_FACTOR: usize = 2;

const KEYCODE_ESC: u8 = 27;
const KEYCODE_ENTER: u8 = 13;
struct Game {
    image: std::sync::Arc<std::sync::Mutex<RgbImage>>,
    keydata_receiver: std::sync::mpsc::Receiver<KeyData>,
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
        self.keydata_receiver.try_recv().ok()
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
    fb.draw_text(
        Point2 {
            x: 100f32,
            y: (1872 / 2) as f32,
        },
        "Loading doom...",
        50f32,
        common::color::BLACK,
        false,
    );
    fb.draw_text(
        Point2 {
            x: 100f32,
            y: (1872 / 2 + 75) as f32,
        },
        "If first started, this will take about a minute.",
        50f32,
        common::color::BLACK,
        false,
    );
    fb.full_refresh(
        common::waveform_mode::WAVEFORM_MODE_GC16,
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

    // Keys
    let key_boxes = [
        (
            common::mxcfb_rect {
                left: 722,
                top: 1400,
                width: 200,
                height: 200 + 10 + 200,
            },
            unsafe { doom::key_left as u8 },
        ),
        (
            common::mxcfb_rect {
                left: 722 + 200 + 10,
                top: 1400,
                width: 200,
                height: 200,
            },
            unsafe { doom::key_up as u8 },
        ),
        (
            common::mxcfb_rect {
                left: 722 + 200 + 10,
                top: 1400 + 200 + 10,
                width: 200,
                height: 200,
            },
            unsafe { doom::key_down as u8 },
        ),
        (
            common::mxcfb_rect {
                left: 722 + 200 + 10 + 200 + 10,
                top: 1400,
                width: 200,
                height: 200 + 10 + 200,
            },
            unsafe { doom::key_right as u8 },
        ),
        (
            common::mxcfb_rect {
                left: 62,
                top: 1400,
                width: 300,
                height: 200 + 10 + 200,
            },
            unsafe { doom::key_strafe as u8 },
        ),
        (
            common::mxcfb_rect {
                left: 62 + 300 + 10,
                top: 1400,
                width: 300,
                height: 200 + 10 + 200,
            },
            unsafe { doom::key_fire as u8 },
        ),
        (
            common::mxcfb_rect {
                left: 62,
                top: 1400 - 10 - 150 - 10 - 150,
                width: 300,
                height: 150,
            },
            KEYCODE_ESC,
        ),
        (
            common::mxcfb_rect {
                left: 62,
                top: 1400 - 150 - 10,
                width: 300,
                height: 150,
            },
            KEYCODE_ENTER,
        ),
        (
            common::mxcfb_rect {
                left: 62 + 300 + 10,
                top: 1400 - 300 - 10 - 10,
                width: 300,
                height: 150 + 10 + 150,
            },
            unsafe { doom::key_use as u8 },
        ),
    ];

    let mut key_labels: fxhash::FxHashMap<u8, (f32, &'static str)> = Default::default();
    key_labels.insert(unsafe { doom::key_left as u8 }, (100.0, "<"));
    key_labels.insert(unsafe { doom::key_up as u8 }, (100.0, "^"));
    key_labels.insert(unsafe { doom::key_down as u8 }, (100.0, "v"));
    key_labels.insert(unsafe { doom::key_right as u8 }, (100.0, ">"));
    key_labels.insert(unsafe { doom::key_strafe as u8 }, (25.0, "Strafe"));
    key_labels.insert(unsafe { doom::key_fire as u8 }, (25.0, "Fire"));
    key_labels.insert(KEYCODE_ESC, (25.0, "ESC"));
    key_labels.insert(KEYCODE_ENTER, (25.0, "Enter"));
    key_labels.insert(unsafe { doom::key_use as u8 }, (25.0, "Use"));

    fb.clear();
    for (boxx, key) in &key_boxes {
        fb.draw_rect(
            Point2 {
                x: boxx.left as i32,
                y: boxx.top as i32,
            },
            Vector2 {
                x: boxx.width,
                y: boxx.height,
            },
            3,
            common::color::BLACK,
        );

        if let Some((key_size, key_label)) = key_labels.get(key) {
            let rect = fb.draw_text(
                Point2 { x: 0f32, y: 500f32 },
                key_label,
                *key_size,
                common::color::BLACK,
                true,
            );
            fb.draw_text(
                Point2 {
                    x: (boxx.left as f32 + (boxx.width - rect.width) as f32 / 2.0),
                    y: (boxx.top as f32 + (boxx.height + rect.height) as f32 / 2.0),
                },
                key_label,
                *key_size,
                common::color::BLACK,
                false,
            );
        }
    }

    fb.full_refresh(
        common::waveform_mode::WAVEFORM_MODE_GC16,
        common::display_temp::TEMP_USE_MAX,
        common::dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
        0,
        true,
    );

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
            //y: (common::DISPLAYHEIGHT as i32 - height as i32) / 2,
            y: 62,
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
                //common::waveform_mode::WAVEFORM_MODE_DU,
                common::waveform_mode::WAVEFORM_MODE_GLR16,
                common::display_temp::TEMP_USE_REMARKABLE_DRAW,
                common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
                0,
                false,
            );

            println!("Waited {:?} to draw image!", start.elapsed());
            last_frame_drawn = Instant::now();
        }
    });

    let (keydata_tx, keydata_rx) = std::sync::mpsc::channel::<doom::KeyData>();

    std::thread::spawn(move || {
        let (input_tx, input_rx) = std::sync::mpsc::channel::<InputEvent>();
        EvDevContext::new(InputDevice::Multitouch, input_tx).start();
        let mut fingers: fxhash::FxHashMap<i32, Finger> = Default::default();
        let mut pressed_keys: fxhash::FxHashSet<u8> = Default::default();

        for event in input_rx {
            match event {
                InputEvent::MultitouchEvent { event } => match event {
                    MultitouchEvent::Press { finger } => {
                        fingers.insert(finger.tracking_id, finger);
                        for keydata in find_updates(&fingers, &key_boxes, &mut pressed_keys) {
                            keydata_tx.send(keydata).ok();
                        }
                    }
                    MultitouchEvent::Move { finger } => {
                        fingers.insert(finger.tracking_id, finger);
                        for keydata in find_updates(&fingers, &key_boxes, &mut pressed_keys) {
                            keydata_tx.send(keydata).ok();
                        }
                    }
                    MultitouchEvent::Release { finger } => {
                        fingers.remove(&finger.tracking_id);
                        for keydata in find_updates(&fingers, &key_boxes, &mut pressed_keys) {
                            keydata_tx.send(keydata).ok();
                        }
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
    });

    doom::init(Game {
        image: image_clone,
        keydata_receiver: keydata_rx,
    });
}

fn find_updates(
    fingers: &fxhash::FxHashMap<i32, Finger>,
    key_boxes: &[(common::mxcfb_rect, u8)],
    pressed_keys: &mut fxhash::FxHashSet<u8>,
) -> Vec<KeyData> {
    let mut events = vec![];
    let last_pressed_keys = pressed_keys.clone();

    pressed_keys.clear();
    for finger in fingers.values() {
        for (boxx, key) in key_boxes {
            if finger.pos.x as u32 >= boxx.left
                && finger.pos.x as u32 <= boxx.left + boxx.width
                && finger.pos.y as u32 >= boxx.top
                && finger.pos.y as u32 <= boxx.top + boxx.height
            {
                pressed_keys.insert(*key);
                break;
            }
        }
    }

    for key_up in last_pressed_keys.difference(&pressed_keys) {
        events.push(KeyData {
            pressed: false,
            key: *key_up,
        });
    }
    for key_down in pressed_keys.difference(&last_pressed_keys) {
        events.push(KeyData {
            pressed: true,
            key: *key_down,
        });
    }

    events
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
