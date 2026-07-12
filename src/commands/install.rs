use crate::filesystem::{read_pkg_info, unpack_package};
use std::collections::{HashMap, HashSet};
use std::fs::{self, create_dir_all, read_dir, read_to_string, write};
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

pub fn parse_pkg_info(text: &str) -> io::Result<Package> {
    let mut name = String::new();
    let mut version = String::new();
    let mut dependencies = Vec::new();
    let mut soname_dependencies = Vec::new();

    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

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
pub fn build_repos_hashmap(
    repo: &str,
    index: &mut HashMap<String, (String, String)>,
) -> io::Result<()> {
    let db_dir = Path::new("/tmp/mirror_list").join(format!("{}_db", repo));
    for entry in read_dir(db_dir)? {
        let entry = entry?.path();
        let desc = entry.join("desc");

        if !desc.exists() {
            continue;
        }

        let mut name = None;
        let mut filename = None;
        let mut section = "";

        for line in read_to_string(&desc)?.lines() {
            match line {
                "%NAME%" => section = "%NAME%",
                "%FILENAME%" => section = "%FILENAME%",
                _ => match section {
                    "%NAME%" if name.is_none() => name = Some(line.to_owned()),

                    "%FILENAME%" if filename.is_none() => filename = Some(line.to_owned()),
                    _ => {}
                },
            }
            if name.is_some() && filename.is_some() {
                break;
            }
        }
        if let (Some(name), Some(filename)) = (name, filename) {
            index.insert(name, (repo.to_string(), filename));
        }
    }
    Ok(())
}
pub fn get_link(index: &HashMap<String, (String, String)>, pkg_name: &str) -> Option<String> {
    index.get(pkg_name).map(|(repo, filename)| {
        format!(
            "https://mirrors.kernel.org/archlinux/{}/os/x86_64/{}",
            repo, filename
        )
    })
}

pub fn is_installed(pkg: &str) -> bool {
    Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_db")
        .join(pkg)
        .exists()
}

pub fn mark_installed(pkg: &str, files: Vec<String>) -> std::io::Result<()> {
    let dir = Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_db").join(pkg);
    create_dir_all(&dir)?;
    write(dir.join("files.txt"), files.join("\n"))
}

pub fn install_pkg(
    index: &HashMap<String, (String, String)>,
    package_name: &str,
    visited: &mut HashSet<String>,
) -> io::Result<()> {
    if !visited.insert(package_name.to_string()) {
        return Ok(());
    }

    if is_installed(package_name) {
        return Ok(());
    }

    match get_link(index, package_name) {
        Some(pkg_link) => {
            let file_name = format!("{}.tar.zst", package_name);
            let output_path = Path::new("/tmp").join(&file_name);

            println!("Downloading {}...", package_name);
            download_file(&pkg_link, &output_path)?;

            let pkg_meta_contents =
                read_pkg_info(&output_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let package = parse_pkg_info(&pkg_meta_contents)?;

            println!("{:?}", &package);

            for dep in package.dependencies {
                let dep_name = dep.split(&['<', '>', '=', ' '][..]).next().unwrap();

                install_pkg(index, dep_name, visited)?;
            }

            let fake_root = Path::new("/home/kiks/Proge/fake-root");

            println!("Unpacking to fake-root...");
            let files = unpack_package(&output_path, fake_root)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            mark_installed(package_name, files);

            let _ = fs::remove_file(output_path);
        }

        None => {
            println!("Pkg '{}' not found in any repo.", package_name);
        }
    }

    Ok(())
}
