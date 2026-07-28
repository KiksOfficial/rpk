use crate::SomeRoot;
use std::fs;

pub fn display_info(pkg_names: &[String], root: &SomeRoot) -> std::io::Result<()> {
    for pkg_name in pkg_names.iter() {
        let path = root
            .root_path
            .join("var/lib/rpk_files")
            .join(pkg_name)
            .join("version.txt");
        let version = fs::read_to_string(path)?;

        println!("{} - {}", &pkg_name, &version.trim());

        let path2 = root.root_path.join("var/lib/rpk_db.txt");
        let deps_rida = fs::read_to_string(path2)?;
        for rida in deps_rida.lines() {
            if let Some((name, deps)) = rida.split_once(":")
                && name == pkg_name
            {
                println!("Dependencies: {}", deps.replace(",", " "));
            }
        }
    }

    Ok(())
}
