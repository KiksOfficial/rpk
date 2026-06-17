use std::fs::{self, read_dir};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn read_metadata(name: String) -> io::Result<String> {
    let contents = fs::read_to_string(name).expect("Should have been able to read the file");
    println!("{}", &contents);
    Ok(contents)
}

pub fn install_package(path: &str) -> io::Result<()> {
    let mut files = Vec::new();

    collect_files(Path::new(path), &mut files)?;
    println!("Files found: {:?}", files);

    Ok(())
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

