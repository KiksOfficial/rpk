mod commands;
mod database;
mod filesystem;
mod package;

use commands::install::read_metadata;

use crate::commands::install::install_package;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package1 = read_metadata(Path::new("/home/kiks/Proge/hello-package/metadata.pkg"))?;
    println!("{:?}", &package1);
    let _ = install_package("/home/kiks/Proge/hello-package");
    Ok(())
}
