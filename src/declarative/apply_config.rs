use std::fs::{self, read_to_string};

use std::path::Path;

struct ParsedConf {
    pkgs: Vec<String>,
    dots: Vec<String>,
}

pub fn read_config(fail: &Path) -> std::io::Result<()> {
    let sisu = read_to_string(fail)?;

    let mut pkgs_mode = false;
    let mut dots_mode = false;

    for rida in sisu.lines() {
        if rida == "[Explicit packages]" {
            pkgs_mode = true;
        }
    }

    Ok(())
}
