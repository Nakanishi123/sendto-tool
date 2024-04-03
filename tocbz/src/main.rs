use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use libtocbz::tocbz;
use rayon::prelude::*;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, io};

fn do_nothing(file: Vec<u8>, name: &Path) -> std::io::Result<(Vec<u8>, PathBuf)> {
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
            tocbz(file, pb, do_nothing);
        });

    println!("Press Enter to exit…");
    let mut input = String::new();
    let stdin = io::stdin();
    let _ = stdin.lock().read_line(&mut input);
}
