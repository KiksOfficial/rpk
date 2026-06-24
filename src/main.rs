mod commands;
mod filesystem;
mod package;

use commands::install::{download_package, read_metadata};
use filesystem::unpack_package;
use package::get_link;
use std::env::args;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argumendid: Vec<String> = args().collect();
    if *&argumendid.len() < 2 {
        println!("Incorrect format (use rpk [option] [package])")
    } /*else {
    if &args[1] == "-S" {
    //download_package(url, output_path)
    todo!()
    };
    }*/
    let package1 = read_metadata(Path::new("/home/kiks/Proge/hello-package/metadata.pkg"))?;
    println!("{:?}", &package1);
    if let Ok(pkg_link) = get_link("fastfetch") {
        let output_path = Path::new("/tmp/fastfetch.tar.gz");
        download_package(&pkg_link, output_path)?;
        let fake_root = Path::new("/home/kiks/Proge/fake-root");
        unpack_package(&output_path, fake_root)?;
    }
    Ok(())
}
