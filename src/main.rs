mod commands;
mod filesystem;
mod package;

use commands::install::{download_package, install_package, read_metadata};
use package::get_link;
use std::env::args;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pkg_url = package::get_link("fastfetch");
    let output_path = Path::new("/tmp/fastfetch.tar.gz");

    match pkg_url {
        Ok(real_pkg_url) => {
            download_package(&real_pkg_url, output_path)?;
        }
        Err(e) => {
            eprintln!("Failed to get package URL: {}", e);
        }
    }
    let argumendid: Vec<String> = args().collect();
    if *&argumendid.len() < 2 {
        println!("Incorrect format (use rpk [option] [package])")
    } /*else {
    if &args[1] == "-S" {
    //download_package(url, output_path)
    todo!()
    };
    }*/
    println!("{:?} balls", &argumendid);
    let package1 = read_metadata(Path::new("/home/kiks/Proge/hello-package/metadata.pkg"))?;
    println!("{:?}", &package1);
    install_package("/home/kiks/Proge/hello-package");
    get_link("fastfetch");
    Ok(())
}
