mod commands;
mod filesystem;
mod handle_diff_errors;

mod unstable;

use crate::commands::install::build_repos_hashmap;
use commands::display_info::display_info;
use commands::install::run_install;
use commands::list::list_installed;
use commands::remove::run_remove;
use commands::update_mirrors::update_mirrors;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::RootKind::{FakeRoot, RealRoot};
use crate::commands::update_packages::run_sys_update;
use unstable::{install_transaction, resolve, run_new_install};

enum RootKind {
    FakeRoot,
    RealRoot,
}

struct SomeRoot {
    pub kind: RootKind,
    pub root_path: PathBuf,
}

fn show_help() {
    eprintln!("Command not found");
    println!(
        "-Sy                updates mirrors\n-S                 downloads packages\n-Syu               download latest mirrors and update all packages\n-R                 remove package ant its dependencies\n-Q                 list packages"
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
        "test" => {
            let mut index = build_repos_hashmap("core")?;
            index.extend(build_repos_hashmap("extra")?);

            let mut visited = HashSet::new();
            let mut graph = HashMap::new();

            resolve(&index, &argumendid[2], &mut visited, &mut graph, &root)?;

            let archives = run_new_install(Arc::new(index), visited)?;

            println!("{:#?}", &graph);
            println!("{:#?}", &archives);

            install_transaction(archives, graph, &root)?;
        }
        _ => {
            eprintln!("Command not found");
            show_help();
        }
    }

    Ok(())
}
