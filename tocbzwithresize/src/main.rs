use image::load_from_memory;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use libtocbz::tocbz;
use rayon::prelude::*;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, io};

const MAX_HEIGHT: u32 = 2560;

fn resize_image(file: Vec<u8>, name: &Path) -> std::io::Result<(Vec<u8>, PathBuf)> {
    let image = match load_from_memory(&file) {
        Ok(image) => image,
        Err(_) => {
            return Ok((file, name.to_path_buf()));
        }
    };
    if MAX_HEIGHT < image.height() {
        let ratio = MAX_HEIGHT as f32 / image.height() as f32;
        let new_width = (image.width() as f32 * ratio) as u32;
        let new_height = MAX_HEIGHT;
        let resized = image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
        let mut buf = std::io::Cursor::new(Vec::new());
        return match resized.write_to(&mut buf, image::ImageFormat::WebP) {
            Ok(_) => {
                let mut new_name = name.to_path_buf();
                new_name.set_extension("webp");
                Ok((buf.into_inner(), new_name))
            }
            Err(_) => panic!("Failed to write image"),
        };
    }
    Ok((file, name.to_path_buf()))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let valid_files: Vec<PathBuf> = args
        .iter()
        .skip(1) // プログラム名をスキップ
        .map(PathBuf::from)
        .collect();

    let m = MultiProgress::new();
    let sty = ProgressStyle::default_spinner()
        .template("{spinner:.blue} {msg}")
        .unwrap()
        .tick_strings(&[
            "▹▹▹▹▹",
            "▸▹▹▹▹",
            "▹▸▹▹▹",
            "▹▹▸▹▹",
            "▹▹▹▸▹",
            "▹▹▹▹▸",
            "▪▪▪▪▪",
        ]);

    let pbs = valid_files
        .iter()
        .map(|file| {
            let pb = m.add(ProgressBar::new_spinner());
            pb.enable_steady_tick(Duration::from_millis(80));
            pb.set_style(sty.clone());
            pb.set_message(format!("準備中: {:?}", file));
            pb
        })
        .collect::<Vec<_>>();

    valid_files
        .par_iter()
        .zip(pbs.par_iter())
        .for_each(|(file, pb)| {
            tocbz(file, pb, resize_image);
        });

    println!("Press Enter to exit…");
    let mut input = String::new();
    let stdin = io::stdin();
    let _ = stdin.lock().read_line(&mut input);
}
