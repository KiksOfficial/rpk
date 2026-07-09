use std::fs::{File, create_dir};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

pub fn unpack_package(src_path: &Path, dest_path: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("tar")
        .args(["-tf", src_path.to_str().ok_or("Invalid source path")?])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Failed to list archive contents".to_string());
    }

    let mut files = String::from_utf8(output.stdout)
        .map_err(|e| e.to_string())?
        .lines()
        .filter_map(|p| {
            if p.starts_with('.') || p.ends_with('/') || p.is_empty() {
                None
            } else {
                Some(p.to_owned())
            }
        })
        .collect::<Vec<_>>();
    files.sort();
    let status = Command::new("tar")
        .args([
            "-xf",
            src_path.to_str().ok_or("Invalid source path")?,
            "-C",
            dest_path.to_str().ok_or("Invalid destination path")?,
        ])
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err("Failed to unpack archive".to_string());
    }

    Ok(files)
}

pub fn register_pkg(name: &str, version: &str, desc: &str, files: &[String]) -> io::Result<()> {
    let db_dir = Path::new("/home/kiks/Proge/fake-root/local-db").join(name);
    if !db_dir.exists() {
        create_dir(&db_dir)?;
    }

    let mut meta_file = File::create(db_dir.join("metadata.txt"))?;
    writeln!(meta_file, "name={}", name)?;
    writeln!(meta_file, "version={}", version)?;
    writeln!(meta_file, "description={}", desc)?;

    let mut files_file = File::create(&db_dir.join("files.txt"))?;

    for file_path in files {
        writeln!(files_file, "{}", file_path)?;
    }
    Ok(())
}
