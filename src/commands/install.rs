use crate::SomeRoot;
use crate::commands::verify_sig::verify_sig;
use crate::filesystem::{read_pkg_info, unpack_package};
use crate::handle_diff_errors::require_args;

use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, read_dir, read_to_string, write};
use std::io::{self};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Package {
    pub name: String,
    pub file_name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub files: Vec<String>,
    pub soname_dependencies: Vec<String>,
}

pub fn parse_pkg_info(text: &str) -> io::Result<Package> {
    let mut name = String::new();
    let mut version = String::new();
    let mut dependencies = Vec::new();
    let mut soname_dependencies = Vec::new();

    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "pkgname" => name = value.to_string(),
                "pkgver" => version = value.to_string(),
                "depend" => {
                    if value.contains(".so=") {
                        soname_dependencies.push(value.to_string());
                    } else {
                        let dep = value.split(['<', '>', '=']).next().unwrap().trim();

                        dependencies.push(dep.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    Ok(Package {
        name,
        file_name: String::new(),
        version,
        dependencies,
        files: Vec::new(),
        soname_dependencies,
    })
}

pub fn download_file(url: &str, output_path: &Path) -> io::Result<()> {
    let path_str = output_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "incorrect file path"))?;

    let status = Command::new("curl")
        .args(["-fsSL", "-o", path_str, url])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "curl gave an error: {:?}",
            status.code()
        )))
    }
}
pub fn build_repos_hashmap(
    repo: &str,
) -> io::Result<HashMap<String, (String, String, String, PathBuf)>> {
    let mut index = HashMap::new();
    let db_dir = Path::new("/tmp/mirror_list").join(format!("{}_db", repo));

    if !db_dir.exists() {
        println!("Directory does not exist!");
        return Ok(index);
    }
    for entry in read_dir(db_dir)? {
        let entry = entry?.path();

        let desc = entry.join("desc");

        let mut name = None;
        let mut filename = None;
        let mut section = "";
        let mut version = None;

        let mut provides = Vec::new();

        for line in read_to_string(&desc)?.lines() {
            match line {
                "" => section = "",
                "%NAME%" => section = "%NAME%",
                "%FILENAME%" => section = "%FILENAME%",
                "%VERSION%" => section = "%VERSION%",

                "%PROVIDES%" => {
                    section = "%PROVIDES%";
                }
                _ => match section {
                    "%NAME%" if name.is_none() => name = Some(line.to_owned()),

                    "%FILENAME%" if filename.is_none() => filename = Some(line.to_owned()),

                    "%VERSION%" if version.is_none() => version = Some(line.to_owned()),
                    "%PROVIDES%" => provides.push(line.to_owned()),
                    _ => {}
                },
            }
        }
        if let (Some(name), Some(filename), Some(version)) = (name, filename, version) {
            let package = (repo.to_string(), filename, version, entry.clone());

            index.insert(name.clone(), package.clone());

            for provide in provides {
                let provide_name = provide.split(['=', '<', '>']).next().unwrap().to_string();

                index.entry(provide_name).or_insert_with(|| package.clone());
            }
        }
    }
    Ok(index)
}
pub fn get_link(
    index: &HashMap<String, (String, String, String, PathBuf)>,
    pkg_name: &str,
) -> Option<String> {
    index.get(pkg_name).map(|(repo, filename, _, _)| {
        format!(
            "https://mirrors.kernel.org/archlinux/{}/os/x86_64/{}",
            repo, filename
        )
    })
}

pub fn is_installed(pkg: &str, root: &SomeRoot) -> bool {
    match read_to_string(root.root_path.join(Path::new("var/lib/rpk_db.txt"))) {
        Ok(db) => db.lines().any(|line| {
            line.split_once(':')
                .map(|(name, _)| name == pkg)
                .unwrap_or(false)
        }),
        Err(_) => false,
    }
}

pub fn mark_installed(
    pkg: &str,
    version: &str,
    files: Vec<String>,
    depends: Vec<String>,
    sonames: Vec<String>,
    root: &SomeRoot,
) -> io::Result<()> {
    let lib_dir = root.root_path.join("var/lib");
    let files_dir = lib_dir.join("rpk_files").join(pkg);
    let db_path = lib_dir.join("rpk_db.txt");

    create_dir_all(&files_dir)?;

    write(files_dir.join("files.txt"), files.join("\n"))?;

    write(files_dir.join("version.txt"), version)?;

    write(files_dir.join("sonames.txt"), sonames.join("\n"))?;

    let mut entries = if db_path.exists() {
        read_to_string(&db_path)?
            .lines()
            .filter(|line| !line.starts_with(&format!("{}:", pkg)))
            .map(String::from)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    entries.push(format!("{}:{}", pkg, depends.join(",")));

    write(db_path, entries.join("\n") + "\n")?;

    println!("Recording {} dependencies: {:?}", pkg, depends);

    Ok(())
}

pub fn get_data_file(
    index: &HashMap<String, (String, String, String, PathBuf)>,
    pkg_name: &str,
) -> io::Result<PathBuf> {
    if let Some((_, _, _, path)) = index.get(pkg_name) {
        return Ok(path.join("desc"));
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

pub fn run_new_install(
    index: Arc<HashMap<String, (String, String, String, PathBuf)>>,
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

                let sig_url = format!("{}.sig", link);
                let sig = PathBuf::from(format!("{}.sig", path.display()));

                download_file(&sig_url, &sig)?;
                download_file(&link, &path)?;

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

    println!("gdk-pixbuf2 = {:?}", index.get("gdk-pixbuf2"));
    println!("glycin = {:?}", index.get("glycin"));
    println!("libglvnd = {:?}", index.get("libglvnd"));
    println!("nvidia-utils = {:?}", index.get("nvidia-utils"));

    resolve(&index, &args[0], &mut visited, &mut graph, root)?;

    let archives = run_new_install(Arc::new(index), visited)?;

    println!("{:#?}", &graph);
    println!("{:#?}", &archives);

    install_transaction(archives, graph, root)?;

    Ok(())
}
