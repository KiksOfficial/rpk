use std::env;
use std::fs::{self, read_to_string};
use std::io::{self, ErrorKind};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::SomeRoot;
use crate::commands::install::run_install;

#[derive(Debug, Default)]
pub struct ParsedConf {
    pub pkgs: Vec<String>,
    pub dots: Vec<String>,
    pub repo: String,
}

enum AppendMode {
    Pkgs,
    Dots,
    Repo,
    None,
}

pub fn read_config(fail: &Path) -> std::io::Result<ParsedConf> {
    let sisu = read_to_string(fail)?;
    let mut current_section = AppendMode::None;
    let mut config = ParsedConf::default();
    for rida in sisu.lines() {
        let trimmed = rida.trim();

        if !trimmed.is_empty() {
            match trimmed {
                "[Explicit packages]" => current_section = AppendMode::Pkgs,
                "[Dotfiles]" => current_section = AppendMode::Dots,
                "[Repo]" => current_section = AppendMode::Repo,
                line => match current_section {
                    AppendMode::Pkgs => config.pkgs.push(line.to_string()),
                    AppendMode::Dots => config.dots.push(line.to_string()),
                    AppendMode::Repo => config.repo = line.to_string(),
                    AppendMode::None => {}
                },
            }
        }
    }

    Ok(config)
}

pub fn build_from_config(obje: &ParsedConf, root: &SomeRoot) -> io::Result<()> {
    run_install(&obje.pkgs, root)?;

    let home = env::var("HOME").expect("Cant get home dir");
    let dotfiles_path = PathBuf::from(&home).join(".dotfiles");
    let status = Command::new("git")
        .args(["clone", &obje.repo])
        .arg(&dotfiles_path)
        .status()?;

    if !status.success() {
        return Err(io::Error::new(ErrorKind::Other, "Failed to clone"));
    }

    let configs_vec = &obje.dots;

    for config in configs_vec {
        let source_path = dotfiles_path.join(config);

        let target_path = PathBuf::from(&home).join(".config").join(config);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if target_path.exists() || target_path.is_symlink() {
            if target_path.is_dir() && !target_path.is_symlink() {
                fs::remove_dir_all(&target_path)?;
            } else {
                fs::remove_file(&target_path)?;
            }
        }

        symlink(&source_path, &target_path)?;
        println!("Symlinked {:?} -> {:?}", target_path, source_path);
    }
    Ok(())
}

pub fn run_use_config(root: &SomeRoot, fail: &String) -> std::io::Result<()> {
    let fail = Path::new(fail);
    let obje = read_config(fail)?;
    build_from_config(&obje, root)?;
    Ok(())
}
