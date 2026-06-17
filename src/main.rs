mod commands;
mod database;
mod filesystem;
mod package;

use commands::install::read_metadata;

use crate::commands::install::install_package;

fn main() {
    let _ = read_metadata("/home/kiks/Proge/hello-package/metadata.toml".to_string());
    let _ = install_package("/home/kiks/Proge/hello-package");
}
