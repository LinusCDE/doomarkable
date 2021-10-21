use doomgeneric::doom::{self, KeyData};
use libremarkable::cgmath::Point2;
use libremarkable::framebuffer::common;
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{FramebufferBase, FramebufferDraw, FramebufferRefresh};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

struct Game<'a> {
    framebuffer: libremarkable::framebuffer::core::Framebuffer<'a>,
    last_frame_drawn: Instant,
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

impl<'a> doom::Doom for Game<'a> {
    fn draw_frame(&mut self, screen_buffer: &[u32], xres: usize, yres: usize) {
        let pos = Point2 {
            x: (common::DISPLAYWIDTH as i32 - xres as i32) / 2,
            y: (common::DISPLAYHEIGHT as i32 - yres as i32) / 2,
        };
        //self.framebuffer.draw_image
        let mut rgb_img = libremarkable::image::RgbImage::new(xres as u32, yres as u32);
        //println!("XRES: {}, YRES: {}", xres, yres);
        /*let mut rgb_img = libremarkable::image::DynamicImage::new_rgb8(
            doom::DOOMGENERIC_RESX as u32,
            doom::DOOMGENERIC_RESY as u32,
        );*/
        assert!(xres * yres == screen_buffer.len());
        for (index, argb) in screen_buffer.iter().enumerate() {
            //println!("X: {}, Y: {}", (index / xres) as u32, (index % xres) as u32);
            rgb_img.put_pixel(
                (index % xres) as u32,
                (index / xres) as u32,
                libremarkable::image::Rgb([
                    ((argb >> 16) & 0xFF) as u8,
                    ((argb >> 8) & 0xFF) as u8,
                    ((argb >> 0) & 0xFF) as u8,
                ]),
            );
        }

        // Useing this to artificially lower the refresh rate on rm2
        if libremarkable::device::CURRENT_DEVICE.model == libremarkable::device::Model::Gen2
            && self.last_frame_drawn.elapsed() < Duration::from_millis(750)
        {
            std::thread::sleep(Duration::from_millis(10));
            return;
        }
        self.framebuffer.draw_image(&rgb_img, pos);
        let start = Instant::now();
        self.framebuffer.partial_refresh(
            &common::mxcfb_rect {
                left: pos.x as u32,
                top: pos.y as u32,
                width: xres as u32,
                height: yres as u32,
            },
            libremarkable::framebuffer::refresh::PartialRefreshMode::Wait,
            common::waveform_mode::WAVEFORM_MODE_GLR16,
            common::display_temp::TEMP_USE_PAPYRUS,
            common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
            0,
            false,
        );
        if start.elapsed() > Duration::from_millis(33) {
            println!("Waited {:?} to draw image!", start.elapsed());
        }
        self.last_frame_drawn = Instant::now();

        /*let mut events = Events::new(EventSettings::new());
        events.set_max_fps(1000);
        while let Some(e) = events.next(&mut self.window) {
            if let Some(button) = e.press_args() {
                if let Some(key) = button_to_doom_key(button) {
                    let keydata = KeyData { pressed: true, key };
                    self.input_queue.push_back(keydata);
                }
            } else if let Some(button) = e.release_args() {
                if let Some(key) = button_to_doom_key(button) {
                    let keydata = KeyData {
                        pressed: false,
                        key,
                    };
                    self.input_queue.push_back(keydata);
                }
            } else if let Some(args) = e.render_args() {
                self.gl.draw(args.viewport(), |c, gl| {
                    // Clear the screen.
                    graphics::clear([0.0, 0.0, 0.0, 1.0], gl);

                    let image = graphics::Image::new().rect([
                        0.0,
                        0.0,
                        f64::from(c.get_view_size()[0]),
                        f64::from(c.get_view_size()[1]),
                    ]);
                    let mut screen_buffer_rgba: Vec<u8> = Vec::with_capacity(xres * yres * 4);
                    for argb in screen_buffer {
                        screen_buffer_rgba.push(((argb >> 16) & 0xFF) as u8);
                        screen_buffer_rgba.push(((argb >> 8) & 0xFF) as u8);
                        screen_buffer_rgba.push(((argb >> 0) & 0xFF) as u8);
                        // Alpha seems to be opacity. Inverting it.
                        screen_buffer_rgba.push(255 - ((argb >> 24) & 0xFF) as u8);
                    }
                    let texture = Texture::create(
                        &mut (),
                        opengl_graphics::Format::Rgba8,
                        &screen_buffer_rgba,
                        [xres as u32, yres as u32],
                        &TextureSettings::new(),
                    )
                    .unwrap();
                    image.draw(&texture, &Default::default(), c.transform, gl);

                    // No image without this useless call!
                    graphics::rectangle(
                        [0.0, 1.0, 0.0, 1.0],
                        graphics::rectangle::square(0.0, 0.0, 0.0),
                        c.transform.trans(0.0, 0.0),
                        gl,
                    );
                });
            } else if let Some(_args) = e.update_args() {
                break;
            }
        }*/
    }
    fn get_key(&mut self) -> Option<doom::KeyData> {
        self.input_queue.pop_front()
    }
    fn set_window_title(&mut self, _title: &str) {
        //self.window.ctx.window().set_title(title);
    }
}

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
    doom::init(Game {
        framebuffer: fb,
        last_frame_drawn: Instant::now() - Duration::from_millis(1000),
        input_queue: VecDeque::new(),
    });
}
