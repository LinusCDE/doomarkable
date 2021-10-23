use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use zstd::{stream::Encoder, DEFAULT_COMPRESSION_LEVEL};

mod blue_noise_calculator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build/");

    //let start = std::time::Instant::now();
    let dither_cache = blue_noise_calculator::calc_full_cache(320, 240);
    //println!("cargo:warning=Calculation took {:?}", start.elapsed());

    let ref f_path = PathBuf::from(env::var("OUT_DIR")?).join("dither_cache.bin.zst");
    let mut f_writer = Encoder::new(File::create(f_path)?, DEFAULT_COMPRESSION_LEVEL)?;
    for val in dither_cache {
        f_writer.write_all(&val.to_le_bytes())?;
    }
    f_writer.finish()?;

    println!(
        "cargo:rustc-env=OUT_DIR_DITHERCACHE_FILE={}",
        f_path.to_str().unwrap()
    );
    Ok(())
}
