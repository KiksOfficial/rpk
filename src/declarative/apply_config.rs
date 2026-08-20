use std::fs::{self, read_to_string};
use std::io;
use std::path::Path;

use crate::SomeRoot;
use crate::commands::install::run_install;

#[derive(Debug, Default)]
struct ParsedConf {
    pkgs: Vec<String>,
    dots: Vec<String>,
    repo: String,
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
    run_install(&obje.pkgs, root);

    Ok(())
}
