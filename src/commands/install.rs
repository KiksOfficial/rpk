use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Package {
    pub name: String,
    pub file_name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub files: Vec<String>,
}

pub fn read_metadata(path: &Path) -> io::Result<Package> {
    let contents = fs::read_to_string(path)?;
    let mut file_name = String::new();
    let mut name = String::new();
    let mut version = String::new();
    let mut dependencies: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    let mut current = String::new();

    let repo_base_url = "https://mirrors.edge.kernel.org/archlinux/core/os/x86_64/";

    println!("{}", &contents);
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('%') && trimmed.ends_with('%') {
            current = trimmed.to_string();
        } else {
            match current.as_str() {
                "%NAME%" => name = trimmed.to_string(),
                "%FILENAME%" => file_name = trimmed.to_string(),
                "%VERSION%" => version = trimmed.to_string(),
                "%DEPENDS%" => dependencies.push(trimmed.to_string()),
                "%FILES%" => files.push(trimmed.to_string()),
                _ => {}
            }
        }
    }

    let _full_url = format!("{}{}", repo_base_url, file_name);
    let package = Package {
        name,
        file_name,
        version,
        dependencies,
        files,
    };

    println!("{:?}", &package);
    Ok(package)
}

pub fn download_file(url: &str, output_path: &Path) -> Result<(), String> {
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

    if !path_obj.exists() {
        let status = Command::new("curl")
            .args(&["-fsSL", "-o", path, db_url])
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Failed to download core.txt".to_string());
        }
    }

    let sisu = fs::read_to_string(path).map_err(|e| e.to_string())?;

    for block in sisu.split("\n\n") {
        if block.is_empty() {
            continue;
        }

        let mut current_name = String::new();
        let mut current_filename = String::new();
        let mut current_section = String::new();

        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('%') && trimmed.ends_with('%') {
                current_section = trimmed.to_string();
            } else {
                match current_section.as_str() {
                    "%NAME%" => current_name = trimmed.to_string(),
                    "%FILENAME%" => current_filename = trimmed.to_string(),
                    _ => {}
                }
            }
        }

        if current_name == pkg_name {
            let repo_base_url = "https://mirrors.edge.kernel.org/archlinux/core/os/x86_64/";
            return Ok(format!("{}{}", repo_base_url, current_filename));
        }
    }

    Err(format!("Package '{}' not found", pkg_name))
}
