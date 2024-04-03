use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use libtocbz::{cbz_name, dir2cbz, rar2cbz, sevenzip2cbz, zip2cbz};
use rayon::prelude::*;
use std::fs::{create_dir_all, rename};
use std::io::BufRead;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, io};

fn tocbz(path: &PathBuf, pb: &ProgressBar) {
    pb.set_message(format!("処理中: {:?}", path));

    let new_name = cbz_name(path);
    let ext = path.extension().unwrap_or_default();
    if path.is_dir() {
        dir2cbz(path, &new_name);
    } else if zip::ZipArchive::new(std::fs::File::open(path).unwrap()).is_ok() {
        zip2cbz(path, &new_name);
    } else if ext == "rar" {
        rar2cbz(path, &new_name);
    } else if ext == "7z" {
        sevenzip2cbz(path, &new_name);
    } else {
        println!("{} is not supported", path.to_str().unwrap());
        return;
    }

    // 完了したファイルをoldディレクトリに移動
    let completed_dir = path.parent().unwrap().join("old");
    let completed_name = completed_dir.join(path.file_name().unwrap());
    if !completed_dir.exists() {
        create_dir_all(&completed_dir).unwrap();
    }
    rename(path, completed_name).unwrap();
    pb.finish_with_message(format!("完了: {:?}", path));
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
            tocbz(file, pb);
        });

    println!("Press Enter to exit…");
    let mut input = String::new();
    let stdin = io::stdin();
    let _ = stdin.lock().read_line(&mut input);
}
