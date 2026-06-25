use std::fs;
use std::path::Path;
use std::process::Command;

pub fn get_link(pkg_name: &str, db: &str) -> Result<String, String> {
    let path = "/home/kiks/Proge/fake-root/core.txt";
    db = "https://raw.githubusercontent.com/KiksOfficial/rpk_db/main/core.txt"

    if !Path::new(path).exists() {
        let status = Command::new("curl")
            .args(&[
                "-fsSL",
                "-o",
                path,
                db,
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
