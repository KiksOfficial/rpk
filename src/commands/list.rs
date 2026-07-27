use std::fs;
use std::io;

use crate::SomeRoot;

pub fn list_installed(root: &SomeRoot) -> io::Result<()> {
    let faili_sisu = fs::read_to_string(root.root_path.join("var/lib/rpk_db.txt"))?;
    for rida in faili_sisu.lines() {
        if let Some(name) = rida.split_once(':') {
            println!("{}", &name.0.trim());
        }
    }
    Ok(())
}
