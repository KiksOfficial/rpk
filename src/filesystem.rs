use std::fs::{self, File, create_dir, read_dir, rename};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn unpack_package(src_path: &Path, dest_path: &Path) -> Result<(), String> {
    let unpack = Command::new("tar")
        .args([
            "-xf",
            src_path.to_str().ok_or("Src path does not exist")?,
            "-C",
            dest_path.to_str().ok_or("Dest path does not exist")?,
        ])
        .status()
        .map_err(|e| e.to_string())?;
    if unpack.success() {
        Ok(())
    } else {
        Err("Failed to unpack".to_string())
    }
}

pub fn register_pkg(name: &str, version: &str, desc: &str, files: &[String]) -> io::Result<()> {
    let db_dir = Path::new("/home/kiks/Proge/fake-root/local-db").join(name);
    create_dir(&db_dir)?;

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
