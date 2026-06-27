mod commands;

use crate::install;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

pub fn update_mirrors() -> io::Result<()> {
    println!("Uuendan peegelsaite...");
    let mirrors_list = ["core"];
    let mirror_url = "https://mirror.archlinux.org";

    for mirror in mirrors_list {
        let url = format!("{}/{}/os/x86_64/{}.db", mirror_url, mirror, mirror);
        let install_path = Path::new("/tmp");
        install::download_file(&url, install_path)?;
    }
    Ok(())
}
