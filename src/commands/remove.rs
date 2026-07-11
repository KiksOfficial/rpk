use std::fs::{remove_dir, remove_dir_all, remove_file};
use std::io;
use std::{fs, path::Path};

pub fn remove_package(pkg_name: &str) -> io::Result<()> {
    let files_txt_path = Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_db")
        .join(pkg_name)
        .join("files.txt");
    let sisu = fs::read_to_string(files_txt_path)?;

    for rida in sisu.lines() {
        let package_location = Path::new("/home/kiks/Proge/fake-root/").join(rida);
        if package_location.exists() {
            if package_location.is_file() {
                remove_file(&package_location)?;
                println!("Deleted file: {:?}", &package_location);
            } else if package_location.is_dir() {
                let _ = remove_dir(&package_location);
            }
        }
    }

    let db_folder = Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_db").join(pkg_name);
    if db_folder.exists() {
        remove_dir_all(db_folder)?;

        println!("Package {} has been deleted", &pkg_name);
    }

    Ok(())
}
