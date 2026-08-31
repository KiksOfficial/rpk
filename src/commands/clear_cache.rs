use crate::SomeRoot;
use std::{
    fs::{self, remove_file},
    path::PathBuf,
};

pub fn clear_cache(root: &SomeRoot) -> std::io::Result<()> {
    let cache_path = PathBuf::from(&root.root_path).join("var/cache/rpk");

    for entry in fs::read_dir(cache_path)? {
        let entry = entry?;
        let file = entry.path();
        if file.is_file() {
            remove_file(file)?;
        }
    }

    Ok(())
}
