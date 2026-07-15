use std::fs::{self, read_dir};
use std::io;
use std::path::Path;

pub fn get_installed_version(pkg_name: &str) -> io::Result<String> {
    let pkg_path = Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_db")
        .join(pkg_name)
        .join("version.txt");
    fs::read_to_string(pkg_path)
}

pub fn get_installed_packages() -> io::Result<Vec<String>> {
    let db = Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_db");

    if !db.exists() {
        return Ok(Vec::new());
    }

    let mut packages = Vec::new();

    for entry in read_dir(db)? {
        let entry = entry?;
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                packages.push(name.to_string());
            }
        }
    }

    Ok(packages)
}
