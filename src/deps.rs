use crate::SomeRoot;
use crate::package::database::is_installed;
use crate::repo::{get_data_file, read_data_into_hashset};
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;

pub fn resolve(
    index: &HashMap<String, (String, String, String, PathBuf)>,
    pkg: &str,
    visited: &mut HashSet<String>,
    graph: &mut HashMap<String, HashSet<String>>,
    root: &SomeRoot,
) -> io::Result<()> {
    let real_pkg = get_real_package_name(index, pkg).unwrap_or_else(|| pkg.to_string());

    if is_installed(&real_pkg, root) {
        return Ok(());
    }

    if !visited.insert(real_pkg.clone()) {
        return Ok(());
    }

    let data_file = get_data_file(index, &real_pkg)?;
    let deps = read_data_into_hashset(&data_file)?;

    let deps = deps
        .into_iter()
        .filter(|dep| !dep.contains(".so=") && dep != "sh")
        .map(|dep| get_real_package_name(index, &dep).unwrap_or(dep))
        .collect::<HashSet<_>>();

    graph.insert(real_pkg.clone(), deps.clone());

    for dep in deps {
        let real_dep = get_real_package_name(index, &dep).unwrap_or(dep);

        resolve(index, &real_dep, visited, graph, root)?;
    }

    Ok(())
}

pub fn clean_graph(graph: &mut HashMap<String, HashSet<String>>) {
    let packages: HashSet<String> = graph.keys().cloned().collect();

    for deps in graph.values_mut() {
        deps.retain(|dep| packages.contains(dep));
    }
}

fn get_real_package_name(
    index: &HashMap<String, (String, String, String, PathBuf)>,
    name: &str,
) -> Option<String> {
    let (_, filename, _, _) = index.get(name)?;

    let filename = filename.strip_suffix(".pkg.tar.zst")?;

    let mut parts = filename.rsplitn(4, '-');

    let _arch = parts.next()?;
    let _pkgrel = parts.next()?;
    let _pkgver = parts.next()?;
    let pkgname = parts.next()?;

    Some(pkgname.to_string())
}
