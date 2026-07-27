mod commands;
mod filesystem;
mod handle_diff_errors;

use commands::display_info::display_info;
use commands::install::run_install;
use commands::list::list_installed;
use commands::remove::run_remove;
use commands::update_mirrors::update_mirrors;
use std::env;
use std::path::{Path, PathBuf};

use crate::RootKind::{FakeRoot, RealRoot};
use crate::commands::update_packages::run_sys_update;

enum RootKind {
    FakeRoot,
    RealRoot,
}

struct SomeRoot {
    pub kind: RootKind,
    pub root_path: PathBuf,
}

impl SomeRoot {
    pub fn path(&self) -> &PathBuf {
        &self.root_path
    }

    pub fn is_fake(&self) -> bool {
        match self.kind {
            RootKind::FakeRoot => true,
            RootKind::RealRoot => false,
        }
    }
}

fn show_help() {
    eprintln!("Command not found");
    println!(
        "-Sy                updates mirrors\n-S                 downloads packages\n-Syu               download latest mirrors and update all packages\n-R                 remove package ant its dependencies\n-Q                 list packages"
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let testing = true;

    let ROOT = if testing {
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
        "-S" => run_install(&argumendid[2..], &ROOT)?,
        "-Syu" => run_sys_update(&ROOT)?,
        "-R" => {
            run_remove(&argumendid[2..], &ROOT)?;
        }

        "-Q" => list_installed(&ROOT)?,
        "-Ss" => display_info(&argumendid[2..], &ROOT)?,
        _ => {
            eprintln!("Command not found");
            show_help();
        }
    }

    Ok(())
}
