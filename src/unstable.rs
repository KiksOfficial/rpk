use crate::SomeRoot;
use crate::commands::install::{
    download_file, get_link, is_installed, mark_installed, parse_pkg_info,
};
use crate::commands::verify_sig::{download_sig, verify_sig};
use crate::filesystem::{read_pkg_info, unpack_package};

use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

pub fn get_data_file(
    index: &HashMap<String, (String, String, String)>,
    pkg_name: &str,
) -> io::Result<PathBuf> {
    if let Some((repo, _filename, version)) = index.get(pkg_name) {
        return Ok(PathBuf::from(format!(
            "/tmp/mirror_list/{}_db/{}-{}/desc",
            repo, pkg_name, version
        )));
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("package {pkg_name} not found"),
    ))
}

pub fn read_data_into_hashset(data_file: &Path) -> io::Result<HashSet<String>> {
    let content = read_to_string(data_file)?;
    let mut deps = HashSet::new();

    let mut in_depends = false;

    for line in content.lines() {
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

    Ok(deps)
}

pub fn resolve(
    index: &HashMap<String, (String, String, String)>,
    pkg: &str,
    visited: &mut HashSet<String>,
    graph: &mut HashMap<String, HashSet<String>>,
    root: &SomeRoot,
) -> io::Result<()> {
    // Do not add installed packages to transaction
    if is_installed(pkg, root) {
        println!("{pkg} already installed");
        return Ok(());
    }

    if !visited.insert(pkg.to_owned()) {
        return Ok(());
    }

    let deps = read_data_into_hashset(&get_data_file(index, pkg)?)?;

    let deps = deps
        .into_iter()
        .filter(|dep| !dep.ends_with(".so") && dep != "sh")
        .collect::<HashSet<_>>();

    graph.insert(pkg.to_owned(), deps.clone());

    for dep in deps {
        resolve(index, &dep, visited, graph, root)?;
    }

    Ok(())
}

pub fn run_new_install(
    index: Arc<HashMap<String, (String, String, String)>>,
    packages: HashSet<String>,
) -> io::Result<HashMap<String, PathBuf>> {
    let handles: Vec<_> = packages
        .into_iter()
        .map(|pkg| {
            let index = Arc::clone(&index);

            thread::spawn(move || -> io::Result<(String, PathBuf)> {
                let link = get_link(&index, &pkg)
                    .ok_or_else(|| io::Error::other(format!("package {pkg} not found")))?;

                let path = PathBuf::from(format!("/tmp/{pkg}.pkg.tar.zst"));

                println!("Downloading {pkg}");

                download_sig(&link, &path)?;
                download_file(&link, &path)?;

                let sig = PathBuf::from(format!("{}.sig", path.display()));

                verify_sig(&sig, &path)?;

                Ok((pkg, path))
            })
        })
        .collect();

    let mut result = HashMap::new();

    for handle in handles {
        let (pkg, path) = handle
            .join()
            .map_err(|_| io::Error::other("download thread panicked"))??;

        result.insert(pkg, path);
    }

    Ok(result)
}

pub fn clean_graph(graph: &mut HashMap<String, HashSet<String>>) {
    let packages: HashSet<String> = graph.keys().cloned().collect();

    for deps in graph.values_mut() {
        deps.retain(|dep| packages.contains(dep));
    }
}

pub fn install_transaction(
    archives: HashMap<String, PathBuf>,
    mut graph: HashMap<String, HashSet<String>>,
    root: &SomeRoot,
) -> io::Result<()> {
    clean_graph(&mut graph);

    let mut remaining = graph;

    while !remaining.is_empty() {
        let ready: Vec<String> = remaining
            .iter()
            .filter(|(_, deps)| deps.iter().all(|dep| !remaining.contains_key(dep)))
            .map(|(pkg, _)| pkg.clone())
            .collect();

        if ready.is_empty() {
            return Err(io::Error::other("dependency cycle detected"));
        }

        for pkg in ready {
            let archive = archives
                .get(&pkg)
                .ok_or_else(|| io::Error::other(format!("missing archive for {pkg}")))?;

            println!("Installing {pkg}");

            let files = unpack_package(archive, &root.root_path).map_err(io::Error::other)?;

            let package = parse_pkg_info(&read_pkg_info(archive).map_err(io::Error::other)?)?;

            mark_installed(&pkg, &package.version, files, package.dependencies, root)?;

            remaining.remove(&pkg);
        }
    }

    Ok(())
}
