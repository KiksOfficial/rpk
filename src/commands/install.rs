use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::commands::update_mirrors::update_mirrors;

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

pub fn get_link(pkg_name: &str, repo_name: &str) -> Result<String, String> {
    let db_dir_path = format!("/tmp/mirror_list/{}_db", repo_name);
    let db_dir = Path::new(&db_dir_path);

    if !db_dir.exists() {
        return Err(format!("Db dir: ({:?}) not found", db_dir));
    }

    let entries = fs::read_dir(db_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.is_dir() {
                let desc_path = path.join("desc");

                if desc_path.exists() {
                    let sisu = fs::read_to_string(&desc_path).map_err(|e| e.to_string())?;

                    let mut current_name = String::new();
                    let mut current_filename = String::new();
                    let mut current_section = String::new();

                    // Parsime desc faili ridu täpselt nii nagu su algses koodis
                    for line in sisu.lines() {
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

                    // Kui leidsime õige paketi nime, tagastame unikaalse allalaadimislingi
                    if current_name == pkg_name {
                        let repo_base_url =
                            format!("https://mirror.archlinux.org/{}/os/x86_64/", repo_name);
                        return Ok(format!("{}{}", repo_base_url, current_filename));
                    }
                }
            }
        }
    }

    Err(format!(
        "Package '{}' not found in repo '{}'",
        pkg_name, repo_name
    ))
}
