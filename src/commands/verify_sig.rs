use std::{
    io::{self, Error, ErrorKind},
    path::{Path, PathBuf},
    process::Command,
};

use crate::commands::install::download_file;

pub fn download_sig(pkg_link: &str, output_path: &Path) -> io::Result<()> {
    let sig_url = format!("{pkg_link}.sig");

    let sig_output = PathBuf::from(format!("{}.sig", output_path.display()));
    println!("{:?}", &sig_output);

    download_file(&sig_url, &sig_output)?;

    Ok(())
}

pub fn verify_sig<'a>(sig_path: &'a Path, pkg_path: &Path) -> std::io::Result<&'a Path> {
    let status = Command::new("gpg")
        .args(["--homedir", "/etc/pacman.d/gnupg", "--verify"])
        .arg(sig_path)
        .arg(pkg_path)
        .status()?;

    if !status.success() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "package signature verification failed",
        ));
    }
    Ok(sig_path)
}
