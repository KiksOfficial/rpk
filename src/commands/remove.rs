use crate::commands::install::is_installed;
use std::collections::{HashMap, HashSet};
use std::fs::{remove_dir, remove_dir_all, remove_file};
use std::io;
use std::{fs, path::Path};

pub fn build_dependency_hashmap(file: &Path) -> io::Result<HashMap<String, Vec<String>>> {
    let sisu = fs::read_to_string(file)?;

    let mut deps: HashMap<String, Vec<String>> = HashMap::new();

    for element in sisu.lines() {
        let Some((package, dependencies)) = element.split_once(':') else {
            continue;
        };

        let list = dependencies
            .split(',')
            .filter(|x| !x.trim().is_empty())
            .map(|x| x.trim().to_string())
            .collect();

        deps.insert(package.trim().to_string(), list);
    }

    Ok(deps)
}

pub fn build_reverse_hashmap(file: &Path) -> io::Result<HashMap<String, Vec<String>>> {
    let sisu = fs::read_to_string(file)?;

    let mut reverse: HashMap<String, Vec<String>> = HashMap::new();

    for element in sisu.lines() {
        let Some((package, dependencies)) = element.split_once(':') else {
            continue;
        };

        for dep in dependencies.split(',') {
            let dep = dep.trim();

            if dep.is_empty() {
                continue;
            }

            let pkg = package.trim().to_string();

            let users = reverse.entry(dep.to_string()).or_default();

            if !users.contains(&pkg) {
                users.push(pkg);
            }
        }
    }

    Ok(reverse)
}

pub fn remove_package_files(pkg_name: &str) -> io::Result<()> {
    let root = Path::new("/home/kiks/Proge/fake-root/");

    let files_txt = root
        .join("var/lib/rpk_files")
        .join(pkg_name)
        .join("files.txt");

    let content = fs::read_to_string(&files_txt).map_err(|e| {
        eprintln!("Failed reading files list {:?}: {}", files_txt, e);
        e
    })?;

    for line in content.lines() {
        let path = root.join(line);

        if path.is_file() {
            remove_file(&path)?;
        } else if path.is_dir() {
            let _ = remove_dir(&path);
        }
    }

    let package_db = root.join("var/lib/rpk_files").join(pkg_name);

    if package_db.exists() {
        remove_dir_all(package_db)?;
    }

    println!("Removed {}", pkg_name);

    Ok(())
}

pub fn remove_package_recursive(
    pkg_name: &str,
    dependencies: &HashMap<String, Vec<String>>,
    reverse: &HashMap<String, Vec<String>>,
    removed: &mut HashSet<String>,
) -> io::Result<()> {
    println!("Entering remove_package_recursive({})", pkg_name);

    if removed.contains(pkg_name) {
        return Ok(());
    }

    let protected = [
        "bash",
        "openssl",
        "systemd-libs",
        "glibc",
        "pam",
        "util-linux-libs",
        "coreutils",
        "filesystem",
        "linux",
        "shadow",
        "rpk",
        "tar",
        "gzip",
        "zstd",
        "xz",
        "grep",
        "sed",
        "awk",
        "ncurses",
    ];
    if protected.contains(&pkg_name) {
        return Ok(());
    }

    removed.insert(pkg_name.to_string());

    if let Some(deps) = dependencies.get(pkg_name) {
        for dep in deps {
            if !is_installed(dep) {
                continue;
            }

            let used_by_other = reverse
                .get(dep)
                .map(|users| {
                    users
                        .iter()
                        .any(|pkg| pkg != pkg_name && is_installed(pkg) && !removed.contains(pkg))
                })
                .unwrap_or(false);

            if !used_by_other {
                remove_package_recursive(dep, dependencies, reverse, removed)?;
            }
        }
    }

    remove_package_files(pkg_name)?;
    remove_package_from_db(pkg_name)?;

    Ok(())
}
pub fn remove_package_from_db(pkg_name: &str) -> io::Result<()> {
    let db_path = Path::new("/home/kiks/Proge/fake-root/var/lib/rpk_db.txt");

    let content = fs::read_to_string(db_path)?;

    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| !line.starts_with(&format!("{}:", pkg_name)))
        .collect();

    fs::write(db_path, filtered.join("\n") + "\n")?;

    Ok(())
}
