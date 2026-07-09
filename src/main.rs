mod commands;
mod filesystem;
mod package;

use commands::install::{build_repos_hashmap, install_pkg};
use commands::update_mirrors::update_mirrors;
use std::collections::{HashMap, HashSet};
use std::env::args;

use crate::commands::remove::remove_package;

pub fn run_install(args: Vec<String>) -> std::io::Result<()> {
    let mut visited = HashSet::new();
    let mut index = HashMap::new();
    build_repos_hashmap("core", &mut index);
    build_repos_hashmap("extra", &mut index);
    println!("Loaded {} packages", &index.len());
    for package in args.iter().skip(2) {
        match install_pkg(&index, package, &mut visited) {
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

    let operation = &argumendid[1];
    if operation == "-Sy" {
        let _ = update_mirrors();
    } else if operation == "-S" {
        run_install(argumendid);
    } else if operation == "-R" {
        remove_package(&argumendid[2]);
    }

    Ok(())
}
