use crate::commands::install;
use crate::filesystem::unpack_package;
use std::fs;
use std::io;
use std::path::Path;

pub fn update_mirrors() -> io::Result<()> {
    println!("Updating mirrors...");
    let mirrors_list = ["core", "extra"];
    let mirror_url = "https://mirror.archlinux.org";

    for mirror in mirrors_list {
        let url = format!("{}/{}/os/x86_64/{}.db", mirror_url, mirror, mirror);
        let install_path = Path::new("/tmp");

        let _ = install::download_file(&url, install_path);

        let src_path = Path::new("/tmp").join(format!("{}.db", mirror));
        let dest_path = Path::new("/tmp/mirror_list").join(format!("{}_db", mirror));

        let _ = unpack_package(&src_path, &dest_path);
    }
    Ok(())
}
