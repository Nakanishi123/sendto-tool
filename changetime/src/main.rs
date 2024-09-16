use filetime_creation::{set_file_times, FileTime};
use std::path::PathBuf;

fn main() {
    let files = get_files_from_args();
    let now = FileTime::now();

    files.iter().for_each(|f| {
        if let Err(e) = set_file_times(f, now, now, now) {
            eprintln!("Failed to set file times for {:?}: {}", f, e);
        } else {
            println!("Set file times for {:?}", f);
        }
    });
    println!("Press Enter to exit…");
    let _ = std::io::stdin().read_line(&mut String::new());
}

fn get_files_from_args() -> Vec<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().skip(1).map(PathBuf::from).collect()
}
