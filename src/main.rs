mod commands;
mod filesystem;

use commands::install::{build_repos_hashmap, install_pkg};
use commands::list::list_installed;
use commands::update_mirrors::update_mirrors;
use commands::update_packages::{get_installed_packages, get_installed_version, update_pkg};
use std::collections::HashSet;
use std::env::args;

pub fn run_install(args: Vec<String>) -> std::io::Result<()> {
    let mut visited = HashSet::new();
    let core = build_repos_hashmap("core")?;
    let extra = build_repos_hashmap("extra")?;
    let mut index = core;
    index.extend(extra);
    println!("Loaded {} packages", &index.len());
    for package in args.iter().skip(2) {
        println!("Trying to install: {}", package);
        println!("{:?}", index.get("htop"));
        match install_pkg(&index, package, &mut visited, false) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}", e)
            }
        }
    }

    Ok(())
}

fn show_help() {
    eprintln!("Command not found");
    println!(
        "-Sy                updates mirrors\n-S                 downloads packages\n-Syu               download latest mirrors and update all packages\n-R                 remove package ant its dependencies\n-Q                 list packages"
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argumendid: Vec<String> = args().collect();
    if argumendid.len() < 2 {
        show_help();
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
            println!("{:?}", &installed);

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
            use crate::commands::remove::{
                build_dependency_hashmap, build_reverse_hashmap, remove_package_recursive,
            };
            use std::collections::HashSet;
            use std::path::Path;

            let db = Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_db.txt");

            println!("Loading DB: {:?}", db);

            let dependencies = match build_dependency_hashmap(db) {
                Ok(x) => {
                    println!("Dependency DB loaded");
                    x
                }
                Err(e) => {
                    eprintln!("Dependency DB failed: {}", e);
                    return Ok(());
                }
            };

            let reverse = match build_reverse_hashmap(db) {
                Ok(x) => {
                    println!("Reverse DB loaded");
                    x
                }
                Err(e) => {
                    eprintln!("Reverse DB failed: {}", e);
                    return Ok(());
                }
            };

            let mut removed = HashSet::new();

            for arg in argumendid.iter().skip(2) {
                println!("Removing {}", arg);

                if let Err(e) = remove_package_recursive(arg, &dependencies, &reverse, &mut removed)
                {
                    eprintln!("Remove failed: {}", e);
                }
            }
        }

        "-Q" => list_installed()?,
        _ => {
            eprintln!("Command not found");
            show_help();
        }
    }

    Ok(())
}
