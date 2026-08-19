use crate::SomeRoot;
use std::fs::{create_dir_all, read_to_string, write};
use std::io::{self};
use std::path::Path;

pub fn is_installed(pkg: &str, root: &SomeRoot) -> bool {
    match read_to_string(root.root_path.join(Path::new("var/lib/rpk_db.txt"))) {
        Ok(db) => db.lines().any(|line| {
            line.split_once(':')
                .map(|(name, _)| name == pkg)
                .unwrap_or(false)
        }),
        Err(_) => false,
    }
}

pub fn mark_installed(
    pkg: &str,
    version: &str,
    files: Vec<String>,
    depends: Vec<String>,
    sonames: Vec<String>,
    root: &SomeRoot,
) -> io::Result<()> {
    let lib_dir = root.root_path.join("var/lib");
    let files_dir = lib_dir.join("rpk_files").join(pkg);
    let db_path = lib_dir.join("rpk_db.txt");

    create_dir_all(&files_dir)?;

    write(files_dir.join("files.txt"), files.join("\n"))?;

    write(files_dir.join("version.txt"), version)?;

    write(files_dir.join("sonames.txt"), sonames.join("\n"))?;

    let mut entries = if db_path.exists() {
        read_to_string(&db_path)?
            .lines()
            .filter(|line| !line.starts_with(&format!("{}:", pkg)))
            .map(String::from)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    entries.push(format!("{}:{}", pkg, depends.join(",")));

    write(db_path, entries.join("\n") + "\n")?;

    println!("Recording {} dependencies: {:?}", pkg, depends);

    Ok(())
}
