use indicatif::ProgressBar;
use std::fs::{create_dir_all, rename, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use unrar::Archive;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

type FileProcessFunc = fn(Vec<u8>, &Path) -> std::io::Result<(Vec<u8>, PathBuf)>;

pub fn is_zip(path: &Path) -> bool {
    zip::ZipArchive::new(std::fs::File::open(path).unwrap()).is_ok()
}

pub fn cbz_name(path: &Path) -> PathBuf {
    let mut new_path = path.to_path_buf();
    if path.is_dir() {
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

pub fn dir2cbz(dir: &PathBuf, new_name: &PathBuf, file_processor: FileProcessFunc) {
    let file = std::fs::File::create(new_name).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let name = path.strip_prefix(dir).unwrap();
            let mut file = std::fs::File::open(path).unwrap();
            let mut buf = Vec::new();
            let _ = file.read_to_end(&mut buf);
            let (buf, new_name) = file_processor(buf, name).unwrap();
            let new_name = new_name.to_str().unwrap().replace('\\', "/");
            zip.start_file(new_name, options).unwrap();
            zip.write_all(&buf).unwrap();
        }
    }
}

pub fn zip2cbz(orig_zip_file: &PathBuf, new_name: &PathBuf, file_processor: FileProcessFunc) {
    let file = std::fs::File::create(new_name).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut archive = ZipArchive::new(File::open(orig_zip_file).unwrap()).unwrap();
    for idex in 0..archive.len() {
        let mut file = archive.by_index(idex).unwrap();
        if file.is_file() {
            let mut buf = Vec::new();
            let _ = file.read_to_end(&mut buf).unwrap();
            let (buf, new_name) = file_processor(buf, Path::new(file.name())).unwrap();
            let new_name = new_name.to_str().unwrap().replace('\\', "/");
            let _ = zip.start_file(new_name, options);
            zip.write_all(&buf).unwrap();
        }
    }
}

pub fn rar2cbz(orig_rar_file: &PathBuf, new_name: &PathBuf, file_processor: FileProcessFunc) {
    let file = std::fs::File::create(new_name).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut archive = Archive::new(orig_rar_file).open_for_processing().unwrap();
    while let Some(header) = archive.read_header().unwrap() {
        if header.entry().is_file() {
            let name = &header.entry().filename.clone();
            let (data, archive_temp) = header.read().unwrap();
            let (buf, new_name) = file_processor(data, name).unwrap();
            let new_name = new_name.to_str().unwrap().replace('\\', "/");
            let _ = zip.start_file(new_name, options);
            zip.write_all(&buf).unwrap();

            archive = archive_temp;
        } else {
            archive = header.skip().unwrap();
        }
    }
}

pub fn sevenzip2cbz(orig_7z_file: &PathBuf, new_name: &PathBuf, file_processor: FileProcessFunc) {
    let binding = tempfile::tempdir_in(orig_7z_file.parent().unwrap()).unwrap();
    let tmp_dir = binding.path();

    sevenz_rust::decompress_file(orig_7z_file, tmp_dir).unwrap();
    dir2cbz(&tmp_dir.to_path_buf(), new_name, file_processor);
}

pub fn tocbz(path: &PathBuf, pb: &ProgressBar, file_processor: FileProcessFunc) {
    pb.set_message(format!("処理中: {:?}", path));

    let new_name = cbz_name(path);
    let ext = path.extension().unwrap_or_default();
    if path.is_dir() {
        dir2cbz(path, &new_name, file_processor);
    } else if is_zip(path) {
        zip2cbz(path, &new_name, file_processor);
    } else if ext == "rar" {
        rar2cbz(path, &new_name, file_processor);
    } else if ext == "7z" {
        sevenzip2cbz(path, &new_name, file_processor);
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
