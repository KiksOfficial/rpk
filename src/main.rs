mod commands;
mod filesystem;
mod package;

use commands::install::{build_repos_hashmap, install_pkg, parse_pkg_info};
use commands::update_mirrors::update_mirrors;
use commands::update_packages::{get_installed_packages, get_installed_version, update_pkg};
use std::collections::{HashMap, HashSet};
use std::env::args;

use crate::commands::remove::remove_package;

pub fn run_install(args: Vec<String>) -> std::io::Result<()> {
    let mut visited = HashSet::new();
    let core = build_repos_hashmap("core")?;
    let extra = build_repos_hashmap("extra")?;
    let mut index = core;
    index.extend(extra);
    println!("Loaded {} packages", &index.len());
    for package in args.iter().skip(2) {
        match install_pkg(&index, package, &mut visited, false) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}", e)
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argumendid: Vec<String> = args().collect();
    if argumendid.len() < 2 {
        println!("Incorrect format (use rpk [option] [package])");
        return Ok(());
    }
    println!("{:?}", &argumendid);
    let operation = &argumendid[1];

    match operation.as_str() {
        "-Sy" => update_mirrors()?,
        "-S" => run_install(argumendid)?,
        "-Syu" => {
            update_mirrors()?;

            let mut index = build_repos_hashmap("core")?;
            let extra = build_repos_hashmap("extra")?;
            index.extend(extra);

            let installed = get_installed_packages()?;

            println!("Installed packages loaded: {}", &installed.len());

            for pkg_name in installed {
                if let Some((_repo, _filename, repo_version)) = index.get(&pkg_name) {
                    let local_version = get_installed_version(&pkg_name)?;

                    if local_version.trim() != repo_version.as_str() {
                        println!(
                            "Upgrade available: {} {} -> {}",
                            pkg_name,
                            local_version.trim(),
                            repo_version
                        );

                        update_pkg(&index, &pkg_name)?;
                    }
                }
            }
        }
        "-R" => {
            for arg in argumendid.iter().skip(2) {
                match remove_package(&arg) {
                    Ok(()) => {}
                    Err(e) => eprintln!("{}", e),
                }
            }
        }
        _ => {
            eprintln!("Command not found")
        }
    }

    Ok(())
}
