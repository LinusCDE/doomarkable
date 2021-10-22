//! This is a slight modification of [mblode](https://github.com/mblode)'s
//! [blue-noise code](https://github.com/mblode/blue-noise/blob/main/src/main.rs)
//! with some small performance improvements and other adjustments.

use libremarkable::image::{GrayImage, ImageBuffer, Luma, RgbImage};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

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

const LATEST_CACHE_FILE_VERSION: u8 = 0x00;
pub struct CachedDither2XTo4X {
    //dither_cache: fxhash::FxHashMap<(Luma<u8>, u32, u32), [Luma<u8>; 16]>,
    dither_cache: Vec<u16>,
}

impl CachedDither2XTo4X {
    pub fn new(width: u32, height: u32) -> Self {
        let mut instance = Self {
            dither_cache: Default::default(),
        };

        if let Some(dither_cache) = Self::read_cache_file(width, height) {
            instance.dither_cache = dither_cache;
        } else {
            instance.calc_full_cache(width, height);
            Self::write_cache_file(&instance.dither_cache);
        }

        instance
    }

    fn cache_file() -> PathBuf {
        PathBuf::from("/home/root/.cache/doomarkable/dither_cache.bin")
    }

    fn read_cache_file(width: u32, height: u32) -> Option<Vec<u16>> {
        if !Self::cache_file().exists() {
            None
        } else {
            let mut reader = BufReader::new(std::fs::File::open(Self::cache_file()).unwrap());
            let mut version = [0xFFu8; 1];
            reader.read_exact(&mut version).unwrap();
            if version[0] != LATEST_CACHE_FILE_VERSION {
                return None;
            }
            let mut dither_cache = vec![0u16; (width as usize / 2) * (height as usize / 2) * 256];
            for i in 0..dither_cache.len() {
                let mut data = [0u8; 2];
                reader.read_exact(&mut data).unwrap();
                dither_cache[i] = u16::from_ne_bytes(data);
            }
            Some(dither_cache)
        }
    }

    fn write_cache_file(dither_cache: &[u16]) {
        std::fs::create_dir_all(Self::cache_file().parent().unwrap()).unwrap();

        let mut writer = BufWriter::new(std::fs::File::create(Self::cache_file()).unwrap());

        writer.write_all(&[LATEST_CACHE_FILE_VERSION]).unwrap();
        for val in dither_cache {
            writer.write_all(&val.to_ne_bytes()).unwrap();
        }
    }

    pub fn calc_full_cache(&mut self, width: u32, height: u32) {
        self.dither_cache = vec![0u16; (width as usize / 2) * (height as usize / 2) * 256];

        // Pre calculate
        for y in 0..(height / 2) {
            for x in 0..(width / 2) {
                for luma in 0..=255 {
                    //let res = instance.calc_dithered_pixels_4x4(&Luma([luma]), x, y);
                    let res = self.calc_dithered_pixels_4x4(&Luma([luma]), x, y);
                    self.dither_cache[Self::calc_dither_cache_index(&Luma([luma]), x, y)] = res;
                }
            }
            println!("Y: {}", y);
        }
    }

    #[inline]
    pub fn calc_dither_cache_index(old_pixel: &Luma<u8>, x: u32, y: u32) -> usize {
        const PIX_WIDTH: usize = 256;
        const LINE_WIDTH: usize = 320 * PIX_WIDTH;
        let x = x as usize;
        let y = y as usize;
        let pix_luma_val = old_pixel.data[0] as usize;
        (y * LINE_WIDTH) + (x * PIX_WIDTH) + pix_luma_val
    }

    #[inline]
    pub fn get_dithered_pixels_4x4(&self, old_pixel: &Luma<u8>, x: u32, y: u32) -> u16 {
        self.dither_cache[Self::calc_dither_cache_index(old_pixel, x, y)]
    }

    #[inline]
    pub fn calc_dithered_pixels_4x4(&self, old_pixel: &Luma<u8>, x: u32, y: u32) -> u16 {
        //let mut res = [Luma([0]); 16];
        let mut res = 0u16;
        let mut i = 0;
        for x_offset in 0..4 {
            for y_offset in 0..4 {
                let wrap_x = wrap(*NOISE_WIDTH, x * 4 + x_offset);
                let wrap_y = wrap(*NOISE_HEIGHT, y * 4 + y_offset);

                let noise_pixel = NOISE_IMG.get_pixel(wrap_x, wrap_y);
                if is_bright(noise_pixel, old_pixel) {
                    //new_img.put_pixel(x_scaled + x_offset, y_scaled + y_offset, Luma([255]));
                    //res[i] = Luma([255]);
                    res |= 0x8000 >> i;
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
        let mut x: u32;
        let mut y: u32 = 0;
        //let mut x_scaled: u32 = 0;
        //let mut y_scaled: u32 = 0;
        //let mut dither_durs = Duration::from_millis(0);
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
                let lhs = ((res >> 8) & 0xFF) as u8;
                let rhs = (res & 0xFF) as u8;
                let expanded: [u8; 16] = [
                    lhs, lhs, lhs, lhs, lhs, lhs, lhs, lhs, rhs, rhs, rhs, rhs, rhs, rhs, rhs, rhs,
                ];
                let rshift: [u8; 16] = [
                    00, 01, 02, 03, 04, 05, 06, 07, 00, 01, 02, 03, 04, 05, 06, 07,
                ];
                // Doesn't improve performance
                let mut expanded = core_simd::u8x16::from_array(expanded);
                let rshift = core_simd::u8x16::from_array(rshift);
                expanded >>= rshift;
                expanded &= 0x01;
                expanded *= 255;

                new_img_vec[i_scaled + 0] = expanded[00];
                new_img_vec[i_scaled + 1] = expanded[01];
                new_img_vec[i_scaled + 2] = expanded[02];
                new_img_vec[i_scaled + 3] = expanded[03];
                let i_scaled_nextline = i_scaled + (width as usize * 4);
                new_img_vec[i_scaled_nextline + 0] = expanded[04];
                new_img_vec[i_scaled_nextline + 1] = expanded[05];
                new_img_vec[i_scaled_nextline + 2] = expanded[06];
                new_img_vec[i_scaled_nextline + 3] = expanded[07];
                let i_scaled_nextline = i_scaled_nextline + (width as usize * 4);
                new_img_vec[i_scaled_nextline + 0] = expanded[08];
                new_img_vec[i_scaled_nextline + 1] = expanded[09];
                new_img_vec[i_scaled_nextline + 2] = expanded[10];
                new_img_vec[i_scaled_nextline + 3] = expanded[11];
                let i_scaled_nextline = i_scaled_nextline + (width as usize * 4);
                new_img_vec[i_scaled_nextline + 0] = expanded[12];
                new_img_vec[i_scaled_nextline + 1] = expanded[13];
                new_img_vec[i_scaled_nextline + 2] = expanded[14];
                new_img_vec[i_scaled_nextline + 3] = expanded[15];

                x += 1;
                //x_scaled += 4;
                i_scaled += 4;
            }

            y += 1;
            //y_scaled += 4;
            i_scaled += width as usize * (4 * 3);
            //println!("Y: {}, Y_Scaled: {}, i_scaled: {}", y, y_scaled, i_scaled);
        }

        //println!("Dither: Calc took {:?}", dither_durs);
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
