use std::fs::File;
use std::fs::{self};
use std::io::{self, Write};
use std::path::PathBuf;

use crate::SomeRoot;

#[derive(Debug)]
struct Config {
    packages: Vec<String>,
    dotfiles: Vec<String>,
    repo: String,
}

pub fn make_config(root: &SomeRoot) -> io::Result<()> {
    let username = std::env::var("USER").map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;

    let home_dir = PathBuf::from("/").join("home").join(&username);

    let config_dir = PathBuf::from(home_dir.join(username).join(".config"));
    let mut config = Config {
        packages: Vec::new(),
        dotfiles: Vec::new(),
        repo: String::new(),
    };

    print!("Enter repo link: ");
    io::stdout().flush()?;

    let mut buffer = String::new();

    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read input");

    config.repo = (*buffer.trim()).to_string();

    if config_dir.is_dir() {
        for entry in fs::read_dir(&config_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(file_name) = path.file_name() {
                config
                    .dotfiles
                    .push(file_name.to_string_lossy().into_owned());
            }
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

    config.dotfiles.sort();
    config.packages.sort();

    let output_path = root.root_path.join("home").join("declarative.txt");

    let mut file = File::create(output_path)?;

    let to_bew_written = format!(
        "[Repo]\n{}\n[Explicit packages]\n{}\n[Dotfiles]\n{}",
        &config.repo,
        &config.packages.join("\n"),
        &config.dotfiles.join("\n")
    );

    writeln!(file, "{}", &to_bew_written)?;

    Ok(())
}
