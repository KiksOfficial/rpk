mod commands;
mod core;
mod filesystem;
mod package;

use commands::install::{download_package, install_package, read_metadata};
use std::env::args;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argumendid: Vec<String> = args().collect();
    if *&argumendid.len() < 2 {
        println!("Incorrect format (use rpk [option] [package])")
    } else {
        if &args[1] == "-S" {
            //download_package(url, output_path)
            todo!()
        };
    }
    println!("{:?} balls", &argumendid);
    let package1 = read_metadata(Path::new("/home/kiks/Proge/hello-package/metadata.pkg"))?;
    println!("{:?}", &package1);
    install_package("/home/kiks/Proge/hello-package");
    Ok(())
}
