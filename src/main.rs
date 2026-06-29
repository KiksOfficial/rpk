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
        /*let mut link_found = None;
        let package_name = &argumendid[2];
        let repos = ["core", "extra"];
        for repo in repos {
            if let Ok(link) = get_link(package_name, repo) {
                link_found = Some(link);
                break;
            }
        }
        match link_found {
            Some(pkg_link) => {
                let file_name = format!("{}.tar.zst", package_name);
                let output_path = Path::new("/tmp").join(&file_name);

                println!("Downloading {}...", package_name);
                download_file(&pkg_link, &output_path)?;

                let fake_root = Path::new("/home/kiks/Proge/fake-root");
                println!("Unpacking to fake-root...");
                unpack_package(&output_path, fake_root)?;

                let metadata_path = fake_root.join(".PKGINFO");
                if metadata_path.exists() {
                    let package1 = pkg_info(&metadata_path)?;
                    println!("{:?}", &package1);
                } else {
                    println!("Package installed successfully (no metadata.pkg found).");
                }
            }
            None => {
                println!("Pkg '{}' not found in any repo.", package_name);
            }
        }*/
    }

    Ok(())
}
