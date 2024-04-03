use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use unrar::Archive;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

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

pub fn dir2cbz(dir: &PathBuf, new_name: &PathBuf) {
    let file = std::fs::File::create(new_name).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let name = path.strip_prefix(dir).unwrap();
            let name = name.to_str().unwrap().replace('\\', "/");
            zip.start_file(name, options).unwrap();
            let mut file = std::fs::File::open(path).unwrap();
            std::io::copy(&mut file, &mut zip).unwrap();
        }
    }
}

pub fn zip2cbz(orig_zip_file: &PathBuf, new_name: &PathBuf) {
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
            zip.write_all(&buf).unwrap();
        }
    }
}

pub fn rar2cbz(orig_rar_file: &PathBuf, new_name: &PathBuf) {
    let file = std::fs::File::create(new_name).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut archive = Archive::new(orig_rar_file).open_for_processing().unwrap();
    while let Some(header) = archive.read_header().unwrap() {
        if header.entry().is_file() {
            let name = &header.entry().filename.to_str().unwrap().replace('\\', "/");
            let (data, archive_temp) = header.read().unwrap();
            let _ = zip.start_file(name, options);
            zip.write_all(&data).unwrap();

            archive = archive_temp;
        } else {
            archive = header.skip().unwrap();
        }
    }
}

pub fn sevenzip2cbz(orig_7z_file: &PathBuf, new_name: &PathBuf) {
    let binding = tempfile::tempdir_in(orig_7z_file.parent().unwrap()).unwrap();
    let tmp_dir = binding.path();

    sevenz_rust::decompress_file(orig_7z_file, tmp_dir).unwrap();
    dir2cbz(&tmp_dir.to_path_buf(), new_name);
}
