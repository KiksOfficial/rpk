mod commands;
mod filesystem;

use commands::install::run_install;
use commands::list::list_installed;
use commands::remove::run_remove;
use commands::update_mirrors::update_mirrors;
use std::env::args;

use crate::commands::update_packages::run_sys_update;

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
        "-Syu" => run_sys_update()?,
        "-R" => {
            run_remove(&argumendid[2..]);
        }

        "-Q" => list_installed()?,
        _ => {
            eprintln!("Command not found");
            show_help();
        }
    }

    Ok(())
}
