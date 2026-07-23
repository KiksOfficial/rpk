use crate::commands::install::{Package, download_file, get_link, mark_installed, parse_pkg_info};
use crate::filesystem::{read_pkg_info, unpack_package};
use std::collections::HashMap;
use std::fs::{self, read_dir};
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub fn get_installed_version(pkg_name: &str) -> io::Result<String> {
    let pkg_path = Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_files")
        .join(pkg_name)
        .join("version.txt");
    fs::read_to_string(pkg_path)
}

pub fn get_installed_packages() -> io::Result<Vec<String>> {
    let db = Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_files");

    if !db.exists() {
        return Ok(Vec::new());
    }

    let mut packages = Vec::new();

    for entry in read_dir(db)? {
        let entry = entry?;
        if entry.path().is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            packages.push(name.to_string());
        }
    }

    Ok(packages)
}

fn fetch_package(
    index: &HashMap<String, (String, String, String)>,
    package_name: &str,
) -> io::Result<(Package, PathBuf)> {
    let pkg_link = get_link(index, package_name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "package not found"))?;

    let output_path = Path::new("/tmp").join(format!("{package_name}.tar.zst"));

    println!("Downloading {}...", package_name);
    download_file(&pkg_link, &output_path)?;

    let pkg_meta = read_pkg_info(&output_path).map_err(io::Error::other)?;
    let package = parse_pkg_info(&pkg_meta)?;

    Ok((package, output_path))
}

pub fn update_pkg(
    index: &HashMap<String, (String, String, String)>,
    package_name: &str,
) -> io::Result<()> {
    let (package, archive) = fetch_package(index, package_name)?;

    println!("Unpacking {}...", package_name);

    let files = unpack_package(&archive, Path::new("/home/kiks/Proge/fake-root"))
        .map_err(io::Error::other)?;

    mark_installed(package_name, &package.version, files, package.dependencies)?;
    fs::remove_file(archive)?;

    Ok(())
}
