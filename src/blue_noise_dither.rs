//! This is a slight modification of [mblode](https://github.com/mblode)'s
//! [blue-noise code](https://github.com/mblode/blue-noise/blob/main/src/main.rs)
//! with some small performance improvements and other adjustments.

use std::{
    ops::Mul,
    time::{Duration, Instant},
};

use libremarkable::image::{GenericImageView, GrayImage, ImageBuffer, Luma, Rgb, RgbImage};

lazy_static::lazy_static! {
    static ref NOISE_IMG: ImageBuffer<Luma<u8>, Vec<u8>> =
        libremarkable::image::load_from_memory(include_bytes!("../img/noise.png")).expect("Load noise.png").grayscale().as_luma8().unwrap().to_owned();
    static ref NOISE_WIDTH: u32 = NOISE_IMG.dimensions().0;
    static ref NOISE_HEIGHT: u32 = NOISE_IMG.dimensions().1;
}

#[inline]
fn is_bright(noise_color: &Luma<u8>, picture_color: &Luma<u8>) -> bool {
    return picture_color.data[0] > noise_color.data[0];
}

#[inline]
fn wrap(m: u32, n: u32) -> u32 {
    return n % m;
}

pub fn dither_image(input_image: &RgbImage, scale_factor: u32) -> RgbImage {
    // RgbImage == ImageBuffer<Rgb<u8>, Vec<u8>>
    let old_img = libremarkable::image::imageops::grayscale(input_image);

    let (width, height) = old_img.dimensions();

    let mut new_img = RgbImage::new(width * scale_factor, height * scale_factor);

    // Using such a naive loop without any additions makes the code about 30% faster!
    let mut x: u32 = 0;
    let mut y: u32 = 0;
    let mut x_scaled: u32 = 0;
    let mut y_scaled: u32 = 0;
    loop {
        if y == height {
            break;
        }

        x = 0;
        x_scaled = 0;
        loop {
            if x == width {
                break;
            }

            let old_pixel = old_img.get_pixel(x, y);

            for x_offset in 0..scale_factor {
                for y_offset in 0..scale_factor {
                    let wrap_x = wrap(*NOISE_WIDTH, x_scaled + x_offset);
                    let wrap_y = wrap(*NOISE_HEIGHT, y_scaled + y_offset);

                    let noise_pixel = NOISE_IMG.get_pixel(wrap_x, wrap_y);
                    if is_bright(noise_pixel, old_pixel) {
                        new_img.put_pixel(
                            x_scaled + x_offset,
                            y_scaled + y_offset,
                            Rgb([255, 255, 255]),
                        );
                    } else {
                        new_img.put_pixel(x_scaled + x_offset, y_scaled + y_offset, Rgb([0, 0, 0]));
                    }
                }
            }

            x += 1;
            x_scaled += scale_factor;
        }

        y += 1;
        y_scaled += scale_factor;
    }
    /*
        for x in 0..width {
            for y in 0..height {
                let old_pixel = old_img.get_pixel(x, y);
                let x_scaled = x * scale_factor;
                let y_scaled = y * scale_factor;

                for x_offset in 0..scale_factor {
                    for y_offset in 0..scale_factor {
                        let wrap_x = wrap(*NOISE_WIDTH, x_scaled + x_offset);
                        let wrap_y = wrap(*NOISE_HEIGHT, y_scaled + y_offset);

                        let noise_pixel = NOISE_IMG.get_pixel(wrap_x, wrap_y);
                        if is_bright(noise_pixel, old_pixel) {
                            new_img.put_pixel(
                                x_scaled + x_offset,
                                y_scaled + y_offset,
                                Rgb([255, 255, 255]),
                            );
                        } else {
                            new_img.put_pixel(x_scaled + x_offset, y_scaled + y_offset, Rgb([0, 0, 0]));
                        }
                    }
                }
            }
        }
    */
    //new_img.save("/tmp/frame.bmp").unwrap();
    new_img
}

pub struct CachedDither2X {
    dither_cache: fxhash::FxHashMap<(Luma<u8>, u32, u32), [Luma<u8>; 4]>,
}

/*
/// TESTING FASTER GRAYSCALER
/// Coefficients to transform from sRGB to a CIE Y (luminance) value.
const SRGB_LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

#[inline]
fn rgb_to_luma(rgb: [u8; 32]) -> [u8; 8] {
    /*let l = SRGB_LUMA[0] * rgb[0].to_f32().unwrap()
        + SRGB_LUMA[1] * rgb[1].to_f32().unwrap()
        + SRGB_LUMA[2] * rgb[2].to_f32().unwrap();
    NumCast::from(l).unwrap()*/
    let a = core_simd::u8x32::from_array(rgb);

    todo!()
}*/

impl CachedDither2X {
    pub fn new(width: u32, height: u32) -> Self {
        let mut instance = Self {
            dither_cache: Default::default(),
        };
        // Pre calculate
        /*for y in 0..height {
            for x in 0..width {
                for luma in 0..255 {
                    instance.calc_dithered_pixels_2x2(&Luma([luma]), y * 2, x * 2);
                }
            }
        }*/
        instance
    }

    pub fn dithered_pixels_2x2(
        &mut self,
        old_pixel: &Luma<u8>,
        x_scaled: u32,
        y_scaled: u32,
    ) -> [Luma<u8>; 4] {
        let key = (*old_pixel, x_scaled, y_scaled);
        if let Some(res) = self.dither_cache.get(&key) {
            *res
        } else {
            let res = self.calc_dithered_pixels_2x2(old_pixel, x_scaled, y_scaled);
            self.dither_cache.insert(key, res);
            res
        }
    }

    #[inline]
    pub fn dithered_pixels_2x2_or_panic(
        &mut self,
        old_pixel: &Luma<u8>,
        x_scaled: u32,
        y_scaled: u32,
    ) -> [Luma<u8>; 4] {
        let key = (*old_pixel, x_scaled, y_scaled);
        *self.dither_cache.get(&key).unwrap()
    }

    pub fn calc_dithered_pixels_2x2(
        &self,
        old_pixel: &Luma<u8>,
        x_scaled: u32,
        y_scaled: u32,
    ) -> [Luma<u8>; 4] {
        let mut res = [Luma([0]); 4];
        let mut i = 0;
        for x_offset in 0..2 {
            for y_offset in 0..2 {
                let wrap_x = wrap(*NOISE_WIDTH, x_scaled + x_offset);
                let wrap_y = wrap(*NOISE_HEIGHT, y_scaled + y_offset);

                let noise_pixel = NOISE_IMG.get_pixel(wrap_x, wrap_y);
                if is_bright(noise_pixel, old_pixel) {
                    //new_img.put_pixel(x_scaled + x_offset, y_scaled + y_offset, Luma([255]));
                    res[i] = Luma([255]);
                } /* else { // Default value
                      //new_img.put_pixel(x_scaled + x_offset, y_scaled + y_offset, Luma([0]));
                      res[i] = Luma([0]);
                  }*/
                i += 1;
            }
        }
        res
    }

    pub fn dither_image(&mut self, input_image: &RgbImage) -> GrayImage {
        // RgbImage == ImageBuffer<Rgb<u8>, Vec<u8>>
        let start = std::time::Instant::now();
        let old_img = libremarkable::image::imageops::grayscale(input_image);
        println!("Dither: Waited {:?} to grayscale image!", start.elapsed());

        let (width, height) = old_img.dimensions();

        //let mut new_img = GrayImage::new(width * 2, height * 2);
        let mut new_img_vec = vec![0u8; (width as usize * 2) * (height as usize * 2)];

        // Using such a naive loop without any additions makes the code about 30% faster!
        let mut i_scaled = 0;
        let mut x: u32 = 0;
        let mut y: u32 = 0;
        let mut x_scaled: u32 = 0;
        let mut y_scaled: u32 = 0;
        let mut dither_durs = Duration::from_millis(0);
        loop {
            if y == height {
                break;
            }

            x = 0;
            x_scaled = 0;
            loop {
                if x == width {
                    break;
                }

                let old_pixel = old_img.get_pixel(x, y);
                //let start = Instant::now();
                //let res = self.calc_dithered_pixels_2x2(old_pixel, x_scaled, y_scaled);
                let res = self.dithered_pixels_2x2(old_pixel, x_scaled, y_scaled);
                //let res = self.dithered_pixels_2x2_or_panic(old_pixel, x_scaled, y_scaled);
                //dither_durs += start.elapsed();
                /*new_img.put_pixel(x_scaled + 0, y_scaled + 0, res[0]);
                new_img.put_pixel(x_scaled + 1, y_scaled + 0, res[1]);
                new_img.put_pixel(x_scaled + 0, y_scaled + 1, res[2]);
                new_img.put_pixel(x_scaled + 1, y_scaled + 1, res[3]);*/
                new_img_vec[i_scaled] = res[0].data[0];
                new_img_vec[i_scaled + 1] = res[1].data[0];
                let i_scaled_nextline = i_scaled + (width as usize * 2);
                new_img_vec[i_scaled_nextline] = res[2].data[0];
                new_img_vec[i_scaled_nextline + 1] = res[3].data[0];

                x += 1;
                x_scaled += 2;
                i_scaled += 2;
            }

            y += 1;
            y_scaled += 2;
            i_scaled += width as usize * 2;
        }

        println!("Dither: Calc took {:?}", dither_durs);
        /*
            for x in 0..width {
                for y in 0..height {
                    let old_pixel = old_img.get_pixel(x, y);
                    let x_scaled = x * scale_factor;
                    let y_scaled = y * scale_factor;

                    for x_offset in 0..scale_factor {
                        for y_offset in 0..scale_factor {
                            let wrap_x = wrap(*NOISE_WIDTH, x_scaled + x_offset);
                            let wrap_y = wrap(*NOISE_HEIGHT, y_scaled + y_offset);

                            let noise_pixel = NOISE_IMG.get_pixel(wrap_x, wrap_y);
                            if is_bright(noise_pixel, old_pixel) {
                                new_img.put_pixel(
                                    x_scaled + x_offset,
                                    y_scaled + y_offset,
                                    Rgb([255, 255, 255]),
                                );
                            } else {
                                new_img.put_pixel(x_scaled + x_offset, y_scaled + y_offset, Rgb([0, 0, 0]));
                            }
                        }
                    }
                }
            }
        */
        //new_img.save("/tmp/frame.bmp").unwrap();
        //new_img
        GrayImage::from_vec(width * 2, height * 2, new_img_vec).unwrap()
    }
}

pub struct CachedDither0XTo4X {
    //dither_cache: fxhash::FxHashMap<(Luma<u8>, u32, u32), [Luma<u8>; 16]>,
    dither_cache: Vec<[Luma<u8>; 16]>,
}

/*
/// TESTING FASTER GRAYSCALER
/// Coefficients to transform from sRGB to a CIE Y (luminance) value.
const SRGB_LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

#[inline]
fn rgb_to_luma(rgb: [u8; 32]) -> [u8; 8] {
    /*let l = SRGB_LUMA[0] * rgb[0].to_f32().unwrap()
        + SRGB_LUMA[1] * rgb[1].to_f32().unwrap()
        + SRGB_LUMA[2] * rgb[2].to_f32().unwrap();
    NumCast::from(l).unwrap()*/
    let a = core_simd::u8x32::from_array(rgb);

    todo!()
}*/

impl CachedDither0XTo4X {
    pub fn new(width: u32, height: u32) -> Self {
        let mut instance = Self {
            dither_cache: vec![[Luma([0]); 16]; (width as usize / 2) * (height as usize / 2) * 256],
        };
        // Pre calculate
        for y in 0..(height / 2) {
            for x in 0..(width / 2) {
                for luma in 0..=255 {
                    //let res = instance.calc_dithered_pixels_4x4(&Luma([luma]), x, y);
                    let res = instance.calc_dithered_pixels_4x4(&Luma([luma]), x, y);
                    instance.dither_cache[Self::calc_dither_cache_index(&Luma([luma]), x, y)] = res;
                }
            }
            println!("Y: {}", y);
        }
        instance
    }

    #[inline]
    pub fn calc_dither_cache_index(old_pixel: &Luma<u8>, x: u32, y: u32) -> usize {
        const pix_width: usize = 256;
        const line_width: usize = 320 * pix_width;
        let x = x as usize;
        let y = y as usize;
        let pix_luma_val = old_pixel.data[0] as usize;
        (y * line_width) + (x * pix_width) + pix_luma_val
    }

    #[inline]
    pub fn get_dithered_pixels_4x4(&self, old_pixel: &Luma<u8>, x: u32, y: u32) -> [Luma<u8>; 16] {
        self.dither_cache[Self::calc_dither_cache_index(old_pixel, x, y)].clone()
    }

    #[inline]
    pub fn calc_dithered_pixels_4x4(&self, old_pixel: &Luma<u8>, x: u32, y: u32) -> [Luma<u8>; 16] {
        let mut res = [Luma([0]); 16];
        let mut i = 0;
        for x_offset in 0..4 {
            for y_offset in 0..4 {
                let wrap_x = wrap(*NOISE_WIDTH, x * 4 + x_offset);
                let wrap_y = wrap(*NOISE_HEIGHT, y * 4 + y_offset);

                let noise_pixel = NOISE_IMG.get_pixel(wrap_x, wrap_y);
                if is_bright(noise_pixel, old_pixel) {
                    //new_img.put_pixel(x_scaled + x_offset, y_scaled + y_offset, Luma([255]));
                    res[i] = Luma([255]);
                } /* else { // Default value
                      //new_img.put_pixel(x_scaled + x_offset, y_scaled + y_offset, Luma([0]));
                      res[i] = Luma([0]);
                  }*/
                i += 1;
            }
        }
        res
    }

    pub fn dither_image(&mut self, input_image: &RgbImage) -> GrayImage {
        // RgbImage == ImageBuffer<Rgb<u8>, Vec<u8>>
        let start = std::time::Instant::now();
        let mut old_img = RgbImage::new(input_image.width() / 2, input_image.height() / 2);
        for y in 0..old_img.height() {
            for x in 0..old_img.width() {
                old_img.put_pixel(x, y, input_image.get_pixel(x * 2, y * 2).clone());
            }
        }
        let old_img = libremarkable::image::imageops::grayscale(&old_img);
        println!("Dither: Waited {:?} to grayscale image!", start.elapsed());

        let (width, height) = old_img.dimensions();
        println!("Width: {}, Height: {}", width, height);

        //let mut new_img = GrayImage::new(width * 2, height * 2);
        let mut new_img_vec = vec![0u8; (width as usize * 4) * (height as usize * 4)];

        // Using such a naive loop without any additions makes the code about 30% faster!
        let mut i_scaled = 0;
        let mut x: u32 = 0;
        let mut y: u32 = 0;
        //let mut x_scaled: u32 = 0;
        //let mut y_scaled: u32 = 0;
        let mut dither_durs = Duration::from_millis(0);
        loop {
            if y == height {
                break;
            }

            x = 0;
            //x_scaled = 0;
            loop {
                if x == width {
                    break;
                }

                let old_pixel = old_img.get_pixel(x, y);
                //let start = Instant::now();
                //let res = self.calc_dithered_pixels_2x2(old_pixel, x_scaled, y_scaled);
                //let res = self.dithered_pixels_4x4(old_pixel, x, y);
                let res = self.get_dithered_pixels_4x4(old_pixel, x, y);
                //let res = self.dithered_pixels_2x2_or_panic(old_pixel, x_scaled, y_scaled);
                //dither_durs += start.elapsed();
                /*new_img.put_pixel(x_scaled + 0, y_scaled + 0, res[0]);
                new_img.put_pixel(x_scaled + 1, y_scaled + 0, res[1]);
                new_img.put_pixel(x_scaled + 0, y_scaled + 1, res[2]);
                new_img.put_pixel(x_scaled + 1, y_scaled + 1, res[3]);*/
                //let i_scaled = (width as usize * 4) * y as usize * 4 + (x as usize * 4);
                new_img_vec[i_scaled] = res[0].data[0];
                new_img_vec[i_scaled + 1] = res[1].data[0];
                new_img_vec[i_scaled + 2] = res[2].data[0];
                new_img_vec[i_scaled + 3] = res[3].data[0];
                let i_scaled_nextline = i_scaled + (width as usize * 4);
                new_img_vec[i_scaled_nextline] = res[4].data[0];
                new_img_vec[i_scaled_nextline + 1] = res[5].data[0];
                new_img_vec[i_scaled_nextline + 2] = res[6].data[0];
                new_img_vec[i_scaled_nextline + 3] = res[7].data[0];
                let i_scaled_nextline = i_scaled_nextline + (width as usize * 4);
                new_img_vec[i_scaled_nextline] = res[8].data[0];
                new_img_vec[i_scaled_nextline + 1] = res[9].data[0];
                new_img_vec[i_scaled_nextline + 2] = res[10].data[0];
                new_img_vec[i_scaled_nextline + 3] = res[11].data[0];
                let i_scaled_nextline = i_scaled_nextline + (width as usize * 4);
                new_img_vec[i_scaled_nextline] = res[12].data[0];
                new_img_vec[i_scaled_nextline + 1] = res[13].data[0];
                new_img_vec[i_scaled_nextline + 2] = res[14].data[0];
                new_img_vec[i_scaled_nextline + 3] = res[15].data[0];

                x += 1;
                //x_scaled += 4;
                i_scaled += 4;
            }

            y += 1;
            //y_scaled += 4;
            i_scaled += width as usize * (4 * 3);
            //println!("Y: {}, Y_Scaled: {}, i_scaled: {}", y, y_scaled, i_scaled);
        }

        println!("Dither: Calc took {:?}", dither_durs);
        /*
            for x in 0..width {
                for y in 0..height {
                    let old_pixel = old_img.get_pixel(x, y);
                    let x_scaled = x * scale_factor;
                    let y_scaled = y * scale_factor;

                    for x_offset in 0..scale_factor {
                        for y_offset in 0..scale_factor {
                            let wrap_x = wrap(*NOISE_WIDTH, x_scaled + x_offset);
                            let wrap_y = wrap(*NOISE_HEIGHT, y_scaled + y_offset);

                            let noise_pixel = NOISE_IMG.get_pixel(wrap_x, wrap_y);
                            if is_bright(noise_pixel, old_pixel) {
                                new_img.put_pixel(
                                    x_scaled + x_offset,
                                    y_scaled + y_offset,
                                    Rgb([255, 255, 255]),
                                );
                            } else {
                                new_img.put_pixel(x_scaled + x_offset, y_scaled + y_offset, Rgb([0, 0, 0]));
                            }
                        }
                    }
                }
            }
        */
        //new_img.save("/tmp/frame.bmp").unwrap();
        //new_img
        GrayImage::from_vec(width * 4, height * 4, new_img_vec).unwrap()
    }
}
