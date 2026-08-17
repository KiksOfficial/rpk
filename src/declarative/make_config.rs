use std::fs::File;
use std::fs::{self, read_to_string};
use std::io::{self, Write};
use std::path::PathBuf;

use crate::SomeRoot;

#[derive(Debug)]
struct Config {
    packages: Vec<String>,
    dotfiles: Vec<Dotfile>,
}

#[derive(Debug)]
struct Dotfile {
    path: PathBuf,
    content: String,
}

pub fn make_config(root: &SomeRoot) -> io::Result<()> {
    let username = std::env::var("USER").map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;

    let temp_config_path = PathBuf::from("/home/kiks");
    //let config_dir = root.root_path.join("home").join(&username).join(".config");

    let config_dir = temp_config_path.join(".config");
    let mut config = Config {
        packages: Vec::new(),
        dotfiles: Vec::new(),
    };
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let content = read_to_string(&path)?;

            config.dotfiles.push(Dotfile { path, content })
        };
    }
    for entry in fs::read_dir(root.root_path.join("var/lib/rpk_files"))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();

        config.packages.push(name);
    }

    let mut file = File::create(root.root_path.join("home/declarative.txt"))?;

    writeln!(file, "packages = {:?}", config.packages)?;
    writeln!(file, "dotfiles = {:?}", config.dotfiles)?;
    Ok(())
}
