use std::env;
use std::fs::{self, read_to_string};
use std::io;
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

    Ok(config)
}

pub fn build__from_config(obje: &ParsedConf, root: &SomeRoot) -> io::Result<()> {
    run_install(&obje.pkgs, root)?;

    let home = env::var("HOME").expect("Cant get home dir");
    let dotfiles_path = PathBuf::from(home).join(".dotfiles");
    Command::new("git")
        .args(["clone", &obje.repo])
        .arg(&dotfiles_path)
        .status()?;

    Ok(())
}
