use std::fs::{self, read_dir, rename};
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
