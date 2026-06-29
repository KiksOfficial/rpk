mod commands;
mod filesystem;
mod package;

use commands::install::{download_file, get_link, install_pkg, pkg_info, read_metadata};
use commands::update_mirrors::update_mirrors;
use filesystem::unpack_package;
use std::collections::HashSet;
use std::env::args;
use std::path::Path;

pub fn run_install(args: Vec<String>) -> std::io::Result<()> {
    let mut visited = HashSet::new();
    install_pkg(&args[2], &mut visited);

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
    }

    Ok(())
}
