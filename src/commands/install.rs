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

pub fn get_link(pkg_name: &str) -> Result<String, String> {
    let path = "/home/kiks/Proge/fake-root/core.txt";

    if !Path::new(path).exists() {
        let status = Command::new("curl")
            .args(&[
                "-fsSL",
                "-o",
                path,
                "https://raw.githubusercontent.com/KiksOfficial/rpk_db/main/core.txt",
            ])
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
