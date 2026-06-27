use crate::commands::install;
use crate::filesystem::unpack_package;
use std::fs;
use std::io;
use std::path::Path;

pub fn update_mirrors() -> io::Result<()> {
    println!("Updating mirrors...");
    let mirrors_list = ["core", "extra"];
    let mirror_url = "https://mirrors.kernel.org/archlinux";

    for mirror in mirrors_list {
        let url = format!("{}/{}/os/x86_64/{}.db", mirror_url, mirror, mirror);

        let src_path = Path::new("/tmp").join(format!("{}.db", mirror));
        let dest_path = Path::new("/tmp/mirror_list").join(format!("{}_db", mirror));

        if src_path.exists() {
            let _ = std::fs::remove_file(&src_path);
        }

        println!("Downloading: {}...", url);
        install::download_file(&url, &src_path)?;

        println!("Unpacking: {:?} -> {:?}", src_path, dest_path);
        std::fs::create_dir_all(&dest_path)?;
        let _ = unpack_package(&src_path, &dest_path);
    }
    Ok(())
}
