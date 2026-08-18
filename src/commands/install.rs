use crate::SomeRoot;
use crate::deps::clean_graph;
use crate::deps::resolve;
use crate::download::run_new_install;
use crate::filesystem::{read_pkg_info, unpack_package};
use crate::handle_diff_errors::require_args;
use crate::package::database::mark_installed;
use crate::package::metadata::parse_pkg_info;
use crate::repo::build_repos_hashmap;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::ErrorKind::FileTooLarge;
use std::io::{self};
use std::path::PathBuf;
use std::sync::Arc;

pub fn install_transaction(
    archives: HashMap<String, PathBuf>,
    mut graph: HashMap<String, HashSet<String>>,
    root: &SomeRoot,
) -> io::Result<()> {
    clean_graph(&mut graph);

    let mut remaining = graph;

    let install_pkg = |pkg: &str| -> io::Result<()> {
        let archive = archives
            .get(pkg)
            .ok_or_else(|| io::Error::other(format!("missing archive for {pkg}")))?;

        println!("Installing {pkg}");

        let files = unpack_package(archive, &root.root_path).map_err(io::Error::other)?;
        let package = parse_pkg_info(&read_pkg_info(archive).map_err(io::Error::other)?)?;
        println!("{:?}", &package);

        mark_installed(
            pkg,
            &package.version,
            files,
            package.dependencies,
            package.soname_dependencies,
            root,
        )?;
        Ok(())
    };

    while !remaining.is_empty() {
        let ready: Vec<String> = remaining
            .iter()
            .filter(|(_, deps)| deps.iter().all(|dep| !remaining.contains_key(dep)))
            .map(|(pkg, _)| pkg.clone())
            .collect();

        if ready.is_empty() {
            println!("Dependency cycle detected, installing remaining packages together:");
            let cycle_pkgs: Vec<String> = remaining.keys().cloned().collect();

            for pkg in &cycle_pkgs {
                println!("  {pkg}");
                install_pkg(pkg)?;
            }
            break;
        }

        for pkg in ready {
            install_pkg(&pkg)?;
            remaining.remove(&pkg);
        }
    }

    Ok(())
}

pub fn run_install(args: &[String], root: &SomeRoot) -> std::io::Result<()> {
    if let Err(e) = require_args(args, 1, "Usage: rpk -S <package> [package...]") {
        eprintln!("{}", e);
        return Ok(());
    }
    let mut index = build_repos_hashmap("core")?;
    index.extend(build_repos_hashmap("extra")?);

    let mut visited = HashSet::new();
    let mut graph = HashMap::new();
    for pkg in args {
        resolve(&index, pkg, &mut visited, &mut graph, root)?;
    }

    let archives = run_new_install(Arc::new(index), visited)?;

    install_transaction(archives, graph, root)?;

    for pkg in args {
        File::create(
            root.root_path
                .join(format!("var/lib/rpk_files/{}/explicit", &pkg)),
        )?;
    }

    Ok(())
}
