use crate::filesystem::register_pkg;
use crate::filesystem::unpack_package;
use std::collections::HashSet;
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
    pub soname_dependencies: Vec<String>,
}

/*pub fn read_metadata(path: &Path) -> io::Result<Package> {
    let contents = fs::read_to_string(path)?;
    let mut file_name = String::new();
    let mut name = String::new();
    let mut version = String::new();
    let mut dependencies: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut soname_dependencies = Vec::new();

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

    let package = Package {
        name,
        file_name,
        version,
        dependencies,
        files,
        soname_dependencies,
    };

    println!("{:?}", &package);
    Ok(package)
}*/

pub fn pkg_info(path: &Path) -> io::Result<Package> {
    let contents = fs::read_to_string(path)?;

    let mut name = String::new();
    let mut version = String::new();
    let mut dependencies = Vec::new();
    let mut soname_dependencies = Vec::new();

    for line in contents.lines() {
        if let Some((key, value)) = line.split_once(" = ") {
            match key {
                "pkgname" => name = value.to_string(),
                "pkgver" => version = value.to_string(),
                "depend" => {
                    if value.contains(".so=") {
                        soname_dependencies.push(value.to_string());
                    } else if value.contains(">=") {
                        continue;
                    } else {
                        dependencies.push(value.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    Ok(Package {
        name,
        file_name: String::new(),
        version,
        dependencies,
        files: Vec::new(),
        soname_dependencies,
    })
}

pub fn download_file(url: &str, output_path: &Path) -> io::Result<()> {
    let path_str = output_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "incorrect file path"))?;

    let status = Command::new("curl")
        .args(&["-fsSL", "-o", path_str, url])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("curl gave an error: {:?}", status.code()),
        ))
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

                    if current_name == pkg_name {
                        let repo_base_url = format!(
                            "https://mirrors.kernel.org/archlinux/{}/os/x86_64/",
                            repo_name
                        );
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

pub fn install_pkg(
    package_name: &str,
    visited: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !visited.insert(package_name.to_string()) {
        return Ok(());
    }

    let mut link_found = None;
    let repos = ["core", "extra"];

    for repo in repos {
        if let Ok(link) = get_link(package_name, repo) {
            link_found = Some(link);
            break;
        }
    }

    match link_found {
        Some(pkg_link) => {
            let file_name = format!("{}.tar.zst", package_name);
            let output_path = Path::new("/tmp").join(&file_name);

            println!("Downloading {}...", package_name);
            download_file(&pkg_link, &output_path)?;

            let fake_root = Path::new("/home/kiks/Proge/fake-root");
            println!("Unpacking to fake-root...");
            unpack_package(&output_path, fake_root)?;

            let metadata_path = fake_root.join(".PKGINFO");

            if metadata_path.exists() {
                let package1 = pkg_info(&metadata_path)?;
                println!("{:?}", &package1);

                for dep in package1.dependencies {
                    let dep_name = dep.split(&['<', '>', '=', ' '][..]).next().unwrap();
                    install_pkg(&dep_name, visited)?;
                }
            } else {
                println!("Package installed successfully (no metadata.pkg found).");
            }
            let _ = fs::remove_file(output_path);
        }

        None => {
            println!("Pkg '{}' not found in any repo.", package_name);
        }
    }

    Ok(())
}
