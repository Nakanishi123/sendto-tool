use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

fn main() {
    let mut files = get_files_from_args();
    files.sort_by_key(|file| file.file_name().unwrap().to_owned());

    let digits = get_digits();
    let start_num = get_start_num();

    let new_names = generate_new_names(&files, digits, start_num);

    if can_rename(&files, &new_names) {
        print_rename_plan(&files, &new_names);

        if confirm_rename() {
            perform_rename(&files, &new_names);
        }
    }

    println!("Press Enter to exit…");
    let mut input = String::new();
    let stdin = io::stdin();
    let _ = stdin.lock().read_line(&mut input);
}

fn get_files_from_args() -> Vec<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().skip(1).map(PathBuf::from).collect()
}

fn get_digits() -> usize {
    let mut digits = String::new();
    loop {
        println!("連番の桁数を入力してください");
        io::stdin().read_line(&mut digits).unwrap();
        match digits.trim().parse::<usize>() {
            Ok(parsed_digits) if parsed_digits > 0 && parsed_digits <= 3 => {
                return parsed_digits;
            }
            _ => {
                println!("1~3の数字を入力してください");
                digits.clear();
            }
        }
    }
}

fn get_start_num() -> usize {
    let mut start_num = String::new();
    loop {
        println!("連番の開始番号を入力してください(デフォルトは1)");
        io::stdin().read_line(&mut start_num).unwrap();
        if start_num.trim().is_empty() {
            return 1;
        }
        match start_num.trim().parse::<usize>() {
            Ok(parsed_num) => {
                return parsed_num;
            }
            _ => {
                println!("数字を入力してください");
                start_num.clear();
            }
        }
    }
}

fn generate_new_names(files: &[PathBuf], digits: usize, start_num: usize) -> Vec<PathBuf> {
    let mut new_names = Vec::new();
    for (i, file) in files.iter().enumerate() {
        let ext = match file.extension() {
            Some(ext) => format!(".{}", ext.to_str().unwrap()),
            None => "".to_string(),
        };
        let new_name = file.with_file_name(format!(
            "{:0digits$}{ext}",
            i + start_num,
            digits = digits,
            ext = ext
        ));
        new_names.push(new_name);
    }
    new_names
}

fn print_rename_plan(files: &[PathBuf], new_names: &[PathBuf]) {
    for (file, new_name) in files.iter().zip(new_names.iter()) {
        println!(
            "{:?} -> {:?}",
            file.file_name().unwrap(),
            new_name.file_name().unwrap()
        );
    }
}

fn can_rename(files: &[PathBuf], new_names: &[PathBuf]) -> bool {
    for new_name in new_names.iter() {
        if new_name.exists() && !files.contains(new_name) {
            println!("{} がダブっています", new_name.display());
            return false;
        }
    }
    true
}

fn confirm_rename() -> bool {
    let mut input = String::new();
    loop {
        println!("リネームしますか？(y/n)");
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "y" => return true,
            "n" => return false,
            _ => {
                println!("yかnを入力してください");
                input.clear();
            }
        }
    }
}

fn perform_rename(files: &[PathBuf], new_names: &[PathBuf]) {
    let temp_paths: Vec<PathBuf> = files
        .iter()
        .map(|path: &std::path::PathBuf| make_temp_file_path(path))
        .collect();
    for (file, temp_path) in files.iter().zip(temp_paths.iter()) {
        std::fs::rename(file, temp_path).unwrap();
    }
    for (temp_path, new_name) in temp_paths.iter().zip(new_names.iter()) {
        std::fs::rename(temp_path, new_name).unwrap();
    }
}

fn make_temp_file_path(file: &Path) -> PathBuf {
    let dir = file.parent().unwrap();
    let file_name = file.file_name().unwrap();
    let mut hasher = DefaultHasher::new();
    file_name.hash(&mut hasher);
    let mut hash = hasher.finish();
    loop {
        hash.hash(&mut hasher);
        hash = hasher.finish();
        let temp_file_name = format!("{}.{:x}", file_name.to_str().unwrap(), hash);
        let temp_file = dir.join(temp_file_name);
        if !temp_file.exists() {
            return temp_file;
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use crate::make_temp_file_path;

    #[test]
    fn test_make_temp_file_path() {
        let file = Path::new("test.txt");
        let temp_file = make_temp_file_path(file);
        println!("{:?}", temp_file);
        assert_ne!(temp_file, file);
    }
}
