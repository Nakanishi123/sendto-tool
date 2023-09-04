use rayon::prelude::*;
use sevenz_rust;
use std::fs::{create_dir_all, metadata, rename, File};
use std::io::{BufRead, Read, Write};
use std::path::PathBuf;
use std::{env, io};
use tempfile;
use unrar::Archive;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

fn cbz_name(path: &PathBuf, is_dir: bool) -> PathBuf {
    let mut new_path = path.clone();
    if is_dir {
        let name = new_path.file_name().unwrap().to_str().unwrap();
        new_path = new_path.with_file_name(format!("{}.cbz", name));
    } else {
        new_path.set_extension("cbz");
    }

    while new_path.exists() {
        let name = new_path.file_stem().unwrap().to_str().unwrap();
        new_path = new_path.with_file_name(format!("{}_new.cbz", name));
    }
    new_path
}

fn dir2cbz(dir: &PathBuf, new_name: &PathBuf) {
    let file = std::fs::File::create(new_name).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let name = path.strip_prefix(dir).unwrap();
            let name = name.to_str().unwrap().replace("\\", "/");
            zip.start_file(name, options).unwrap();
            let mut file = std::fs::File::open(path).unwrap();
            std::io::copy(&mut file, &mut zip).unwrap();
        }
    }
}

fn zip2cbz(orig_zip_file: &PathBuf, new_name: &PathBuf) {
    let file = std::fs::File::create(new_name).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut archive = ZipArchive::new(File::open(orig_zip_file).unwrap()).unwrap();
    for idex in 0..archive.len() {
        let mut file = archive.by_index(idex).unwrap();
        if file.is_file() {
            let mut buf = Vec::new();
            let _ = file.read_to_end(&mut buf);
            let _ = zip.start_file(file.name(), options);
            let _ = zip.write_all(&buf).unwrap();
        }
    }
}

fn rar2cbz(orig_rar_file: &PathBuf, new_name: &PathBuf) {
    let file = std::fs::File::create(new_name).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut archive = Archive::new(orig_rar_file).open_for_processing().unwrap();
    while let Some(header) = archive.read_header().unwrap() {
        if header.entry().is_file() {
            let name = &header.entry().filename.to_str().unwrap().replace("\\", "/");
            let (data, archive_temp) = header.read().unwrap();
            let _ = zip.start_file(name, options);
            let _ = zip.write_all(&data).unwrap();

            archive = archive_temp;
        } else {
            archive = header.skip().unwrap();
        }
    }
}

fn sevenzip2cbz(orig_7z_file: &PathBuf, new_name: &PathBuf) {
    let binding = tempfile::tempdir_in(orig_7z_file.parent().unwrap()).unwrap();
    let tmp_dir = binding.path();

    sevenz_rust::decompress_file(orig_7z_file, tmp_dir).unwrap();
    dir2cbz(&tmp_dir.to_path_buf(), new_name);
}

fn tocbz(path: &PathBuf) {
    if path.is_dir() {
        let new_name = cbz_name(path, true);
        dir2cbz(path, &new_name);
        return;
    }

    let new_name = cbz_name(path, path.is_dir());
    let ext = path.extension().unwrap();
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
    println!(
        "{} -> {}",
        path.to_str().unwrap(),
        new_name.to_str().unwrap()
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let valid_files: Vec<PathBuf> = args
        .iter()
        .skip(1) // プログラム名をスキップ
        .filter_map(|arg| {
            let path = PathBuf::from(arg);
            if metadata(&path).is_ok() {
                Some(path)
            } else {
                panic!("{} is not a valid path", arg)
            }
        })
        .collect();

    valid_files.par_iter().for_each(|file| {
        tocbz(file);
    });

    println!("Press Enter to exit…");
    let mut input = String::new();
    let stdin = io::stdin();
    let _ = stdin.lock().read_line(&mut input);
}
