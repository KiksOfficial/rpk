mod commands;
mod declarative;
mod deps;
mod download;
mod filesystem;
mod handle_diff_errors;
mod package;
mod repo;
mod unstable;

use commands::display_info::display_info;
use commands::install::run_install;
use commands::list::list_installed;
use commands::remove::run_remove;
use commands::update_mirrors::update_mirrors;
use declarative::make_config::make_config;
use std::env;
use std::path::PathBuf;

use crate::commands::update_packages::run_sys_update;

enum RootKind {
    FakeRoot,
    RealRoot,
}

#[warn(dead_code)]
struct SomeRoot {
    pub kind: RootKind,
    pub root_path: PathBuf,
}

fn show_help() {
    eprintln!("Command not found");
    println!(
        "-Sy                updates mirrors\n-S                 downloads packages\n-Syu               download latest mirrors and update all packages\n-R                 remove package and its dependencies\n-Q                 list packages\n -Ss                 display pkg info"
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let testing = true;

    let root = if testing {
        SomeRoot {
            kind: RootKind::FakeRoot,
            root_path: PathBuf::from("/home/kiks/Proge/fake-root/"),
        }
    } else {
        SomeRoot {
            kind: RootKind::RealRoot,
            root_path: PathBuf::from("/"),
        }
    };

    let argumendid: Vec<String> = env::args().collect();
    if argumendid.len() < 2 {
        show_help();
        return Ok(());
    }
    println!("{:?}", &argumendid);
    let operation = &argumendid[1];

    match operation.as_str() {
        "-Sy" => update_mirrors()?,
        "-S" => run_install(&argumendid[2..], &root)?,
        "-Syu" => run_sys_update(&root)?,
        "-R" => {
            run_remove(&argumendid[2..], &root)?;
        }

        "-Q" => list_installed(&root)?,
        "-Ss" => display_info(&argumendid[2..], &root)?,
        "--make-config" => make_config(&root)?,
        "test" => {
            println!("For testing unstable stuff")
        }
        _ => {
            eprintln!("Command not found");
            show_help();
        }
    }

    Ok(())
}
