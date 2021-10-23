//#![feature(portable_simd)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[macro_use]
extern crate log;

use doomgeneric::{game, game::DoomGeneric, input::keys, input::KeyData};
use libremarkable::cgmath::{Point2, Vector2};
use libremarkable::framebuffer::common;
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{
    refresh::PartialRefreshMode, FramebufferBase, FramebufferDraw, FramebufferIO,
    FramebufferRefresh,
};
use libremarkable::image::{DynamicImage, RgbImage};
use libremarkable::input::{
    ev::EvDevContext, multitouch::Finger, multitouch::MultitouchEvent, InputDevice, InputEvent,
};
use std::io::Cursor;
use std::time::{Duration, Instant};

mod blue_noise_dither;

const SCALE_FACTOR: usize = 2;

struct Game {
    image: std::sync::Arc<std::sync::Mutex<RgbImage>>,
    keydata_receiver: std::sync::mpsc::Receiver<KeyData>,
}

impl DoomGeneric for Game {
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
    fn get_key(&mut self) -> Option<KeyData> {
        self.keydata_receiver.try_recv().ok()
    }
    fn set_window_title(&mut self, _title: &str) {
        //self.indow.ctx.window().set_title(title);
    }
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();

    let mut fb = Framebuffer::from_path("/dev/fb0");
    let mut preparing_text_rect = fb.draw_text(
        Point2 {
            x: 600f32,
            y: (1872 / 2) as f32,
        },
        "Preparing...",
        50f32,
        common::color::BLACK,
        false,
    );
    preparing_text_rect.left -= 50;
    preparing_text_rect.top -= 50;
    preparing_text_rect.width += 50 * 2;
    preparing_text_rect.height += 50 * 2;
    fb.partial_refresh(
        &preparing_text_rect,
        PartialRefreshMode::Wait,
        common::waveform_mode::WAVEFORM_MODE_GC16_FAST,
        common::display_temp::TEMP_USE_AMBIENT,
        common::dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
        0,
        true,
    );

    // The dither_cache was calculated in build/main.rs and
    // this env is set to the file path containing this cache.
    let start = Instant::now();
    let dither_cache_compressed = include_bytes!(env!("OUT_DIR_DITHERCACHE_FILE"));
    let mut dither_cache_raw = Cursor::new(Vec::with_capacity(
        blue_noise_dither::CachedDither2XTo4X::calc_dither_cache_len(320, 240) * 2,
    ));
    zstd::stream::copy_decode(Cursor::new(dither_cache_compressed), &mut dither_cache_raw).unwrap();
    let dither_cache_raw = dither_cache_raw.into_inner();
    let mut ditherer = blue_noise_dither::CachedDither2XTo4X::new(dither_cache_raw);
    info!("Loaded dither cache in {:?}", start.elapsed());

    // Keys
    let key_boxes = [
        (
            common::mxcfb_rect {
                left: 722,
                top: 1400,
                width: 200,
                height: 200 + 10 + 200,
            },
            *keys::KEY_LEFT,
        ),
        (
            common::mxcfb_rect {
                left: 722 + 200 + 10,
                top: 1400,
                width: 200,
                height: 200,
            },
            *keys::KEY_UP,
        ),
        (
            common::mxcfb_rect {
                left: 722 + 200 + 10,
                top: 1400 + 200 + 10,
                width: 200,
                height: 200,
            },
            *keys::KEY_DOWN,
        ),
        (
            common::mxcfb_rect {
                left: 722 + 200 + 10 + 200 + 10,
                top: 1400,
                width: 200,
                height: 200 + 10 + 200,
            },
            *keys::KEY_RIGHT,
        ),
        (
            common::mxcfb_rect {
                left: 62,
                top: 1400,
                width: 300,
                height: 200 + 10 + 200,
            },
            *keys::KEY_STRAFE,
        ),
        (
            common::mxcfb_rect {
                left: 62 + 300 + 10,
                top: 1400,
                width: 300,
                height: 200 + 10 + 200,
            },
            *keys::KEY_FIRE,
        ),
        (
            common::mxcfb_rect {
                left: 62,
                top: 1400 - 10 - 150 - 10 - 150,
                width: 300,
                height: 150,
            },
            keys::KEY_ESCAPE,
        ),
        (
            common::mxcfb_rect {
                left: 62,
                top: 1400 - 150 - 10,
                width: 300,
                height: 150,
            },
            keys::KEY_ENTER,
        ),
        (
            common::mxcfb_rect {
                left: 62 + 300 + 10,
                top: 1400 - 300 - 10 - 10,
                width: 300,
                height: 150 + 10 + 150,
            },
            *keys::KEY_USE,
        ),
    ];

    let mut key_labels: fxhash::FxHashMap<u8, (f32, &'static str)> = Default::default();
    key_labels.insert(*keys::KEY_LEFT, (100.0, "<"));
    key_labels.insert(*keys::KEY_UP, (100.0, "^"));
    key_labels.insert(*keys::KEY_DOWN, (100.0, "v"));
    key_labels.insert(*keys::KEY_RIGHT, (100.0, ">"));
    key_labels.insert(*keys::KEY_STRAFE, (25.0, "Strafe"));
    key_labels.insert(*keys::KEY_FIRE, (25.0, "Fire"));
    key_labels.insert(keys::KEY_ESCAPE, (25.0, "ESC"));
    key_labels.insert(keys::KEY_ENTER, (25.0, "Enter"));
    key_labels.insert(*keys::KEY_USE, (25.0, "Use"));

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
        game::DOOMGENERIC_RESX as u32,
        game::DOOMGENERIC_RESY as u32,
    )));
    let image_clone = image.clone();
    std::thread::spawn(move || {
        let mut last_frame_drawn = Instant::now() - Duration::from_millis(1000);
        let width = game::DOOMGENERIC_RESX as u32 * SCALE_FACTOR as u32;
        let height = game::DOOMGENERIC_RESY as u32 * SCALE_FACTOR as u32;
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
            // Downscale 2x (doomgeneric does a simple upscale anyways, so no data lost)
            // TODO: Remove need for downscaling in doomgeneric-rs
            let rgb_img = RgbImage::from_fn(rgb_img.width() / 2, rgb_img.height() / 2, |x, y| {
                *rgb_img.get_pixel(x * 2, y * 2)
            });

            let dithered_img = ditherer.dither_image(&DynamicImage::ImageRgb8(rgb_img));
            debug!("Dithering took {:?}", start.elapsed());

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
                PartialRefreshMode::Async,
                //common::waveform_mode::WAVEFORM_MODE_DU,
                common::waveform_mode::WAVEFORM_MODE_GLR16,
                common::display_temp::TEMP_USE_REMARKABLE_DRAW,
                common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
                0,
                false,
            );

            debug!("Drawing took {:?}", start.elapsed());
            last_frame_drawn = Instant::now();
        }
    });

    let (keydata_tx, keydata_rx) = std::sync::mpsc::channel::<KeyData>();

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

    game::init(Game {
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
    let width = img.width();
    let height = img.height();
    let mut fb_raw_data: Vec<u8> =
        Vec::with_capacity(img.width() as usize * 2 * img.height() as usize);
    let img_vec = img.to_vec();
    for pixel_value in img_vec {
        fb_raw_data.push(pixel_value);
        fb_raw_data.push(pixel_value);
    }

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
}
