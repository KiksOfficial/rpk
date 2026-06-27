mod commands;
mod filesystem;
mod package;

use commands::install::{download_file, get_link, read_metadata};
use filesystem::unpack_package;
use std::env::args;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argumendid: Vec<String> = args().collect();
    if argumendid.len() < 2 {
        println!("Incorrect format (use rpk [option] [package])");
        return Ok(());
    } /*else {
    if &args[1] == "-S" {
    //download_package(url, output_path)
    todo!()
    };
    }*/
    let db = "https://raw.githubusercontent.com/KiksOfficial/rpk_db/main/core.txt";
    let _ = update_mirrors();
    if let Ok(pkg_link) = get_link("fastfetch", db) {
        let output_path = Path::new("/tmp/fastfetch.tar.gz");
        download_file(&pkg_link, output_path)?;

        let fake_root = Path::new("/home/kiks/Proge/fake-root");

        unpack_package(&output_path, fake_root)?;
        let metadata_path = fake_root.join("metadata.pkg");

        let package1 = read_metadata(&metadata_path)?;

        println!("{:?}", &package1);
    }
    Ok(())
}
