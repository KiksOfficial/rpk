use crate::commands::install::build_repos_hashmap;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::io;
use std::path::{Path, PathBuf};

pub fn get_data_file(pkg_name: &str) -> std::io::Result<PathBuf> {
    let mut core = build_repos_hashmap("core")?;
    let extra = build_repos_hashmap("extra")?;
    let index = &mut core;

    index.extend(extra);

    if let Some(ennik) = core.get(pkg_name) {
        let full_pkg_path = PathBuf::from(format!(
            "/tmp/mirror_list/{}_db/{}-{}/desc",
            &ennik.0, &pkg_name, &ennik.2
        ));
        println!("{:?}", &full_pkg_path);
        return Ok(full_pkg_path);
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "package not found"))
}

pub fn read_data_into_hashset(data_file: &Path) -> io::Result<HashSet<String>> {
    let sisu = read_to_string(data_file)?;
    let mut deps = HashSet::new();

    let mut in_depends = false;

    for line in sisu.lines() {
        match line {
            "%DEPENDS%" => {
                in_depends = true;
                continue;
            }
            l if l.starts_with('%') => {
                in_depends = false;
                continue;
            }
            _ => {}
        }

        if in_depends && !line.is_empty() {
            let dep = line.split(['<', '>', '=']).next().unwrap().trim();

            deps.insert(dep.to_string());
        }
    }
    println!("{:?}", &deps);

    Ok(deps)
}

pub fn resolve(pkg: &str, visited: &mut HashSet<String>) -> io::Result<()> {
    if !visited.insert(pkg.to_string()) {
        return Ok(());
    }

    println!("Processing {pkg}");

    let path = get_data_file(pkg)?;
    let deps = read_data_into_hashset(&path)?;

    for dep in deps {
        if dep.ends_with(".so") {
            continue;
        }
        resolve(&dep, visited)?;
    }

    Ok(())
}
