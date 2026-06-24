use std::fs::{self, read_dir, rename};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

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

pub fn download_package(url: &str, output_path: &Path) -> Result<(), String> {
    let downloading = Command::new("curl")
        .args([
            "-fsSL",
            "-s",
            "-o",
            output_path.to_str().ok_or("Invalid output path")?,
            url,
        ])
        .status()
        .map_err(|e| e.to_string())?;

    if downloading.success() {
        Ok(())
    } else {
        Err("Download failed".to_string())
    }
}

pub fn unpack_package(src_path: &Path, dest_path: &Path) -> Result<(), String> {
    let unpack = Command::new("tar")
        .args([
            "-xf",
            src_path.to_str().ok_or("Src path does not exist")?,
            "-C",
            dest_path.to_str().ok_or("Dest path does not exist")?,
        ])
        .status()
        .map_err(|e| e.to_string())?;
    if unpack.success() {
        Ok(())
    } else {
        Err("Failed to unpack".to_string())
    }
}
