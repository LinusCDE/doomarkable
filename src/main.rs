//#![feature(portable_simd)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[macro_use]
extern crate log;

use doomgeneric::{game, game::DoomGeneric, input::KeyData};
use libremarkable::cgmath::Point2;
use libremarkable::framebuffer::common;
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{
    refresh::PartialRefreshMode, FramebufferBase, FramebufferDraw, FramebufferIO,
    FramebufferRefresh,
};
use libremarkable::image::{DynamicImage, RgbImage};
use libremarkable::input::{ev::EvDevContext, InputDevice, InputEvent};
use once_cell::sync::Lazy;
use std::io::Cursor;
use std::sync::Mutex;
use std::time::{Duration, Instant};

mod blue_noise_dither;
mod layout;

const SCALE_FACTOR: usize = 2;
pub static FB: Lazy<Mutex<Framebuffer>> =
    Lazy::new(|| Mutex::new(Framebuffer::from_path("/dev/fb0")));

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

    // Ensure .savegame and wad file are always relative to the home directory
    std::env::set_current_dir("/home/root").unwrap();

    let mut preparing_text_rect = FB.lock().unwrap().draw_text(
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
    FB.lock().unwrap().partial_refresh(
        &preparing_text_rect,
        PartialRefreshMode::Wait,
        common::waveform_mode::WAVEFORM_MODE_GC16_FAST,
        common::display_temp::TEMP_USE_AMBIENT,
        common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
        0,
        true,
    );
    FB.lock().unwrap().clear();

    // The dither_cache was calculated in build/main.rs and
    // this env is set to the file path containing this cache.
    let start = Instant::now();
    let dither_cache_compressed = include_bytes!(env!("OUT_DIR_DITHERCACHE_FILE"));
    let mut dither_cache_raw = Cursor::new(Vec::with_capacity(
        blue_noise_dither::CachedDither2XTo4X::calc_dither_cache_len(
            game::DOOMGENERIC_RESX as u32 / 2,
            game::DOOMGENERIC_RESY as u32 / 2,
        ) * 2,
    ));
    zstd::stream::copy_decode(Cursor::new(dither_cache_compressed), &mut dither_cache_raw).unwrap();
    let dither_cache_raw = dither_cache_raw.into_inner();
    let mut ditherer = blue_noise_dither::CachedDither2XTo4X::new(dither_cache_raw);
    info!("Loaded dither cache in {:?}", start.elapsed());

    // Title
    let title_text = concat!("DOOMarkable v", env!("CARGO_PKG_VERSION"));
    let subtitle_text = "https://github.com/LinusCDE/doomarkable";
    let title_size = 80;
    let subtitle_size = 30;
    let title_rect = FB.lock().unwrap().draw_text(
        Point2 { x: 0f32, y: 0f32 },
        title_text,
        title_size as f32,
        common::color::BLACK,
        true,
    );
    let subtitle_rect = FB.lock().unwrap().draw_text(
        Point2 { x: 0f32, y: 0f32 },
        subtitle_text,
        subtitle_size as f32,
        common::color::BLACK,
        true,
    );

    FB.lock().unwrap().draw_text(
        Point2 {
            x: (common::DISPLAYWIDTH as u32 - title_rect.width) as f32 / 2.0,
            y: (62 - 20 + title_size) as f32,
        },
        title_text,
        title_size as f32,
        common::color::BLACK,
        false,
    );
    FB.lock().unwrap().draw_text(
        Point2 {
            x: (common::DISPLAYWIDTH as u32 - subtitle_rect.width) as f32 / 2.0,
            y: (62 - 20 + title_size + subtitle_size) as f32,
        },
        subtitle_text,
        subtitle_size as f32,
        common::color::BLACK,
        false,
    );

    // Keys

    FB.lock().unwrap().full_refresh(
        common::waveform_mode::WAVEFORM_MODE_GC16,
        common::display_temp::TEMP_USE_MAX,
        common::dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
        0,
        true,
    );

    let default_image =
        libremarkable::image::load_from_memory(include_bytes!("../res/default_screen.png"))
            .unwrap()
            .to_rgb();
    let image = std::sync::Arc::new(std::sync::Mutex::new(default_image));
    let image_clone = image.clone();
    std::thread::spawn(move || {
        let mut last_frame_drawn = Instant::now() - Duration::from_millis(1000);
        let width = game::DOOMGENERIC_RESX as u32 * SCALE_FACTOR as u32;
        let height = game::DOOMGENERIC_RESY as u32 * SCALE_FACTOR as u32;
        let pos = Point2 {
            x: (common::DISPLAYWIDTH as i32 - width as i32) / 2,
            //y: (common::DISPLAYHEIGHT as i32 - height as i32) / 2,
            y: 62 + 140,
        };
        let max_fps = match libremarkable::device::CURRENT_DEVICE.model {
            // Will probably not quite hit these anyways
            libremarkable::device::Model::Gen1 => 15,
            // The rM 2 "can" do more, but will result in async frames and more lag. Won't be anymore fluid anyways.
            libremarkable::device::Model::Gen2 => 3,
        };
        let frame_duration = Duration::from_micros(1000000 / max_fps);

        let battery_indicator_update_interval = Duration::from_secs(30);
        let mut last_battery_indicator_update = Instant::now() - battery_indicator_update_interval;
        let mut last_battery_percentage = -99;

        loop {
            // Limit fps
            let elapsed = last_frame_drawn.elapsed();
            if elapsed < frame_duration {
                //debug!("Hitting max fps!!!");
                let remaining = frame_duration - elapsed;
                if remaining <= Duration::from_millis(2) {
                    std::thread::yield_now();
                } else {
                    std::thread::sleep(remaining - Duration::from_millis(1));
                }
                continue;
            }

            // Battery indicator in corner
            if last_battery_indicator_update.elapsed() > battery_indicator_update_interval {
                last_battery_indicator_update = Instant::now();
                let percentage = libremarkable::battery::percentage().unwrap_or(-1);
                if percentage != last_battery_percentage {
                    last_battery_percentage = percentage;

                    let text = format!("{}%    ", percentage); // Spaces to prevent residual text when text gets narrower
                    let rect = FB.lock().unwrap().draw_text(
                        Point2 {
                            x: 10.0,
                            y: (common::DISPLAYHEIGHT - 10) as f32,
                        },
                        &text,
                        30f32,
                        common::color::BLACK,
                        false,
                    );
                    FB.lock().unwrap().partial_refresh(
                        &rect,
                        PartialRefreshMode::Async,
                        common::waveform_mode::WAVEFORM_MODE_GC16_FAST,
                        common::display_temp::TEMP_USE_MAX,
                        common::dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
                        0,
                        false,
                    );
                    debug!("Updated battery indicator");
                }
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
            draw_image_mono(&mut FB.lock().unwrap(), pos, &dithered_img);

            let waveform = match libremarkable::device::CURRENT_DEVICE.model {
                libremarkable::device::Model::Gen1 => common::waveform_mode::WAVEFORM_MODE_GLR16,
                libremarkable::device::Model::Gen2 => common::waveform_mode::WAVEFORM_MODE_DU,
            };
            FB.lock().unwrap().partial_refresh(
                &common::mxcfb_rect {
                    left: pos.x as u32,
                    top: pos.y as u32,
                    width,
                    height,
                },
                PartialRefreshMode::Async,
                //common::waveform_mode::WAVEFORM_MODE_DU,
                //common::waveform_mode::WAVEFORM_MODE_GLR16,
                waveform,
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
        let mut layout_manager = layout::LayoutManager::new(&mut FB.lock().unwrap());

        let (input_tx, input_rx) = std::sync::mpsc::channel::<InputEvent>();
        EvDevContext::new(InputDevice::Multitouch, input_tx).start();

        for event in input_rx {
            for outcome in layout_manager.current_layout_mut().handle_input(event) {
                match outcome {
                    layout::InputOutcome::KeyData(keydata) => {
                        keydata_tx.send(keydata).ok();
                    }
                    layout::InputOutcome::SwitchLayout(new_layout_id) => {
                        layout_manager.switch_layout(new_layout_id, &mut FB.lock().unwrap())
                    }
                }
            }
        }
    });

    game::init(Game {
        image: image_clone,
        keydata_receiver: keydata_rx,
    });
    // TODO: Doom hogs the entire cpu when failed to start (no wad file).
    // Need to figure out how to trigger on error.
    warn!("Game loop quit!");
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
