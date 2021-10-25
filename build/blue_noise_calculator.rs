//! This is a slight modification of [mblode](https://github.com/mblode)'s
//! [blue-noise code](https://github.com/mblode/blue-noise/blob/main/src/main.rs)
//! with some small performance improvements and other adjustments.
//! The base code as and noise.png are licensed under MIT:
//! https://github.com/mblode/blue-noise/blob/568d18f5/LICENSE.md

use image::{ImageBuffer, Luma};
use once_cell::sync::Lazy;

static NOISE_IMG: Lazy<ImageBuffer<Luma<u8>, Vec<u8>>> = Lazy::new(|| {
    image::load_from_memory(include_bytes!("noise.png"))
        .expect("Load noise.png")
        .grayscale()
        .as_luma8()
        .unwrap()
        .to_owned()
});
static NOISE_WIDTH: Lazy<u32> = Lazy::new(|| NOISE_IMG.width());
static NOISE_HEIGHT: Lazy<u32> = Lazy::new(|| NOISE_IMG.height());

#[inline]
fn is_bright(noise_color: &Luma<u8>, picture_color: &Luma<u8>) -> bool {
    return picture_color[0] > noise_color[0];
}

#[inline]
fn wrap(m: u32, n: u32) -> u32 {
    return n % m;
}

#[inline]
fn calc_dithered_pixels_4x4(old_pixel: &Luma<u8>, x: u32, y: u32) -> u16 {
    // Increase brightness 1.5x
    let ref old_pixel = Luma([(((old_pixel[0] as f32 / 255.0) * 1.5) * 255.0) as u8]);

    let mut res = 0u16;
    let mut i = 0;
    for y_offset in 0..4 {
        for x_offset in 0..4 {
            let wrap_x = wrap(*NOISE_WIDTH, x * 4 + x_offset);
            let wrap_y = wrap(*NOISE_HEIGHT, y * 4 + y_offset);

            let noise_pixel = NOISE_IMG.get_pixel(wrap_x, wrap_y);
            if is_bright(noise_pixel, old_pixel) {
                res |= 0x8000 >> i;
            } // else default value (0 bit)
            i += 1;
        }
    }
    res
}

/// TODO: Fix duplication in src/blue_noise_dither.rs
#[inline]
fn calc_dither_cache_len(width: u32, height: u32) -> usize {
    width as usize * height as usize * 256
}

/// TODO: Fix duplication in src/blue_noise_dither.rs
#[inline]
fn calc_dither_cache_index(old_pixel: &Luma<u8>, x: u32, y: u32, width: u32) -> usize {
    const PIX_WIDTH: usize = 256; // 256 shades of gray (each with its own dithered u16)
    let line_width: usize = width as usize * PIX_WIDTH;
    let x = x as usize;
    let y = y as usize;
    let pix_luma_val = old_pixel[0] as usize;
    (y * line_width) + (x * PIX_WIDTH) + pix_luma_val
}

/// The u16 contains a 4x4 array of pixel bits (1 = black, 0 = white)
pub fn calc_full_cache(width: u32, height: u32) -> Vec<u16> {
    let mut dither_cache = vec![0u16; calc_dither_cache_len(width, height)];

    // Pre calculate
    for y in 0..height {
        for x in 0..width {
            for luma in 0..=255 {
                //let res = instance.calc_dithered_pixels_4x4(&Luma([luma]), x, y);
                let res = calc_dithered_pixels_4x4(&Luma([luma]), x, y);
                dither_cache[calc_dither_cache_index(&Luma([luma]), x, y, width)] = res;
            }
        }
        //debug!("Y: {}", y);
    }

    dither_cache
}
