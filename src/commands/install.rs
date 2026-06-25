use std::fs::{self};
use std::io::{self};
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub files: Vec<String>,
}

pub fn read_metadata(path: &Path) -> io::Result<Package> {
    let contents = fs::read_to_string(path)?;
    let mut name = String::new();
    let mut version = String::new();
    let mut dependencies: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();

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
                "file" => files.push(value.to_string()),
                _ => {}
            }
        }
    }
    let package = Package {
        name,
        version,
        dependencies,
        files,
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

pub fn get_link(pkg_name: &str, db_url: &str) -> Result<String, String> {
    let path = "/home/kiks/Proge/sync/core.txt";
    let path_obj = Path::new(path);

    if let Some(parent) = path_obj.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if !Path::new(path).exists() {
        let status = Command::new("curl")
            .args(&["-fsSL", "-o", path, db_url])
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Failed to download core.txt".to_string());
        }
    }

    let sisu = fs::read_to_string(path).map_err(|e| e.to_string())?;

    for line in sisu.lines() {
        let mut parts = line.split_whitespace();

        if let Some(name) = parts.next() {
            if name == pkg_name {
                if let Some(url) = parts.next() {
                    return Ok(url.to_string());
                }
            }
        }
    }

    Err(format!("Package '{}' not found", pkg_name))
}
