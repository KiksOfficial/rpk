use crate::commands::install;
use crate::filesystem::unpack_package;
use std::fs;
use std::io;
use std::path::Path;

pub fn update_mirrors() -> io::Result<()> {
    println!("Updating mirrors...");

    let mirrors = ["core", "extra"];
    let base_url = "https://mirrors.kernel.org/archlinux";

    for mirror in mirrors {
        let url = format!("{}/{}/os/x86_64/{}.db", base_url, mirror, mirror);

        let src_path = Path::new("/tmp").join(format!("{mirror}.db"));
        let dest_path = Path::new("/tmp/mirror_list").join(format!("{mirror}_db"));

        if dest_path.exists() {
            fs::remove_dir_all(&dest_path)?;
        }
        fs::create_dir_all(&dest_path)?;

        if src_path.exists() {
            fs::remove_file(&src_path)?;
        }

        println!("Downloading {url}...");
        install::download_file(&url, &src_path)?;

        println!("Unpacking {:?} -> {:?}", src_path, dest_path);
        let _ = unpack_package(&src_path, &dest_path);

        fs::remove_file(&src_path)?;
    }

    Ok(())
}
