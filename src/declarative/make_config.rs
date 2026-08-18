use std::fs::File;
use std::fs::{self};
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
}

pub fn make_config(root: &SomeRoot) -> io::Result<()> {
    let username = std::env::var("USER").map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;

    let home_dir = PathBuf::from("/").join("home").join(&username);
    //let config_dir = home_dir.join(".config");
    //
    let config_dir = PathBuf::from("/home/kiks/.config");
    let mut config = Config {
        packages: Vec::new(),
        dotfiles: Vec::new(),
    };

    if config_dir.is_dir() {
        for entry in fs::read_dir(&config_dir)? {
            let entry = entry?;
            let path = entry.path();

            let relative = path.strip_prefix(&home_dir).map_err(io::Error::other)?;

            config.dotfiles.push(Dotfile {
                path: relative.to_path_buf(),
            });
        }
    }

    let packages_dir = root.root_path.join("var/lib/rpk_files");

    if packages_dir.is_dir() {
        for entry in fs::read_dir(&packages_dir)? {
            let entry = entry?;

            if entry.path().join("explicit").exists() {
                let name = entry.file_name().to_string_lossy().into_owned();

                config.packages.push(name);
            }
        }
    }

    let output_path = root.root_path.join("home").join("declarative.txt");

    let mut file = File::create(output_path)?;

    let to_bew_written = format!(
        "[Explicit packages]\n{}\n[Dotfiles]\n{}",
        &config.packages.join("\n"),
        &config
            .dotfiles
            .iter()
            .map(|dotfile| dotfile.path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );

    writeln!(file, "{}", &to_bew_written)?;

    Ok(())
}
