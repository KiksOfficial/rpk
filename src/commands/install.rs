use std::fs::{self, read_dir, rename};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

pub fn read_metadata(path: &Path) -> io::Result<Package> {
    let contents = fs::read_to_string(path)?;
    let mut name = String::new();
    let mut version = String::new();
    let mut dependencies: Vec<String> = Vec::new();

    println!("{}", &contents);
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let mut parts = trimmed.splitn(2, "=");
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();

            match key {
                "name" => name = value.to_string(),
                "version" => version = value.to_string(),
                "dependency" => dependencies.push(value.to_string()),
                _ => {}
            }
        }
    }
    let package = Package {
        name,
        version,
        dependencies,
    };

    println!("{:?}", &package);
    Ok(package)
}

pub fn install_package(path: &str) -> io::Result<()> {
    let mut files = Vec::new();
    let base_path = Path::new("/home/kiks/Proge/");

    let package_path = Path::new(path);
    let package_name = package_path.file_name().unwrap();

    let rootfs = package_path.join("rootfs");

    let install_root = base_path.join(Path::new("fake-root").join(package_name));

    collect_files(&rootfs, &mut files)?;

    for file in &files {
        if let Ok(relative) = file.strip_prefix(&rootfs) {
            let dest = install_root.join(relative);

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::rename(file, dest)?;
        }
    }

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
