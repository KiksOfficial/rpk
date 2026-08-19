use std::fs;
use std::io;

use crate::SomeRoot;

pub fn list_installed(root: &SomeRoot, explicit: bool) -> io::Result<()> {
    let faili_sisu = fs::read_to_string(root.root_path.join("var/lib/rpk_db.txt"))?;
    for rida in faili_sisu.lines() {
        if let Some(name) = rida.split_once(':') {
            let pkg_path = root
                .root_path
                .join(format!("var/lib/rpk_files/{}/explicit", &name.0.trim()));
            if pkg_path.exists() && explicit {
                println!("{}", &name.0.trim());
            }

            if !explicit {
                println!("{}", &name.0.trim());
            }
        }
    }
    Ok(())
}
