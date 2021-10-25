use libremarkable::image::{DynamicImage, GenericImageView, GrayImage, Luma};

pub struct CachedDither2XTo4X {
    dither_cache: Vec<u16>,
}

impl CachedDither2XTo4X {
    fn convert_vec_u8_to_vec_u16(vec: Vec<u8>) -> Vec<u16> {
        assert!(vec.len() % 2 == 0);
        vec.chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect()
    }

    pub fn new(raw_dither_cache: Vec<u8>) -> Self {
        Self {
            dither_cache: Self::convert_vec_u8_to_vec_u16(raw_dither_cache),
        }
    }

    /// TODO: Fix duplication in build/blue_noise_calculator.rs
    #[inline]
    fn calc_dither_cache_index(old_pixel: &Luma<u8>, x: u32, y: u32) -> usize {
        const PIX_WIDTH: usize = 256; // 256 shades of gray (each with its own dithered u16)
        const LINE_WIDTH: usize = 320 as usize * PIX_WIDTH; // TODO: 320 is still hardcoded!
        let x = x as usize;
        let y = y as usize;
        let pix_luma_val = old_pixel[0] as usize;
        (y * LINE_WIDTH) + (x * PIX_WIDTH) + pix_luma_val
    }

    /// TODO: Fix duplication in build/blue_noise_calculator.rs
    #[inline]
    pub fn calc_dither_cache_len(width: u32, height: u32) -> usize {
        width as usize * height as usize * 256
    }

    #[inline]
    pub fn get_dithered_pixels_4x4(&self, old_pixel: &Luma<u8>, x: u32, y: u32) -> u16 {
        self.dither_cache[Self::calc_dither_cache_index(old_pixel, x, y)]
    }

    pub fn dither_image(&mut self, input_image: &DynamicImage) -> GrayImage {
        assert_eq!(
            Self::calc_dither_cache_len(input_image.width(), input_image.height()),
            self.dither_cache.len()
        );

        // RgbImage == ImageBuffer<Rgb<u8>, Vec<u8>>
        let start = std::time::Instant::now();
        // Grayscale image
        //let old_img = libremarkable::image::imageops::grayscale(input_image);
        // No need for srgb correction.
        let old_img = libremarkable::image::GrayImage::from_fn(
            input_image.width(),
            input_image.height(),
            |x, y| {
                let pixel = input_image.get_pixel(x, y);
                let r = pixel.data[0] as u16;
                let g = pixel.data[1] as u16;
                let b = pixel.data[2] as u16;
                Luma([((r + g + b) / 3) as u8])
            },
        );
        debug!("Dither: Grayscaling took {:?}", start.elapsed());

        let (width, height) = old_img.dimensions();

        //let mut new_img = GrayImage::new(width * 4, height * 4);
        let mut new_img_vec = vec![0u8; (width as usize * 4) * (height as usize * 4)];

        // Using such a naive loop without any additions makes the code about 30% faster!
        let mut i_scaled = 0;
        let mut x: u32;
        let mut y: u32 = 0;
        loop {
            if y == height {
                break;
            }

            x = 0;
            loop {
                if x == width {
                    break;
                }

                let old_pixel = old_img.get_pixel(x, y);
                let res = self.get_dithered_pixels_4x4(old_pixel, x, y);
                new_img_vec[i_scaled + 0] = ((res >> 00) & 0x1) as u8 * 255;
                new_img_vec[i_scaled + 1] = ((res >> 01) & 0x1) as u8 * 255;
                new_img_vec[i_scaled + 2] = ((res >> 02) & 0x1) as u8 * 255;
                new_img_vec[i_scaled + 3] = ((res >> 03) & 0x1) as u8 * 255;
                let i_scaled_nextline = i_scaled + (width as usize * 4);
                new_img_vec[i_scaled_nextline + 0] = ((res >> 04) & 0x1) as u8 * 255;
                new_img_vec[i_scaled_nextline + 1] = ((res >> 05) & 0x1) as u8 * 255;
                new_img_vec[i_scaled_nextline + 2] = ((res >> 06) & 0x1) as u8 * 255;
                new_img_vec[i_scaled_nextline + 3] = ((res >> 07) & 0x1) as u8 * 255;
                let i_scaled_nextline = i_scaled_nextline + (width as usize * 4);
                new_img_vec[i_scaled_nextline + 0] = ((res >> 08) & 0x1) as u8 * 255;
                new_img_vec[i_scaled_nextline + 1] = ((res >> 09) & 0x1) as u8 * 255;
                new_img_vec[i_scaled_nextline + 2] = ((res >> 10) & 0x1) as u8 * 255;
                new_img_vec[i_scaled_nextline + 3] = ((res >> 11) & 0x1) as u8 * 255;
                let i_scaled_nextline = i_scaled_nextline + (width as usize * 4);
                new_img_vec[i_scaled_nextline + 0] = ((res >> 12) & 0x1) as u8 * 255;
                new_img_vec[i_scaled_nextline + 1] = ((res >> 13) & 0x1) as u8 * 255;
                new_img_vec[i_scaled_nextline + 2] = ((res >> 14) & 0x1) as u8 * 255;
                new_img_vec[i_scaled_nextline + 3] = ((res >> 15) & 0x1) as u8 * 255;

                x += 1;
                i_scaled += 4;
            }

            y += 1;
            i_scaled += width as usize * (4 * 3);
        }

        GrayImage::from_vec(width * 4, height * 4, new_img_vec).unwrap()
    }
}
