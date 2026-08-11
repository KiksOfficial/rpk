use crate::SomeRoot;

use crate::commands::update_mirrors::update_mirrors;
use crate::commands::verify_sig::{download_sig, verify_sig};
use crate::download::download_file;
use crate::filesystem::{read_pkg_info, unpack_package};
use crate::package::database::mark_installed;
use crate::package::metadata::Package;
use crate::package::metadata::parse_pkg_info;
use crate::repo::{build_repos_hashmap, get_link};

use std::collections::HashMap;
use std::fs::{self, read_dir};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;

pub fn get_installed_version(pkg_name: &str, root: &SomeRoot) -> io::Result<String> {
    let pkg_path = root
        .root_path
        .join("var/lib/rpk_files")
        .join(pkg_name)
        .join("version.txt");
    fs::read_to_string(pkg_path)
}

pub fn get_installed_packages(root: &SomeRoot) -> io::Result<Vec<String>> {
    let db = root.root_path.join("var/lib/rpk_files");

    if !db.exists() {
        return Ok(Vec::new());
    }

    let mut packages = Vec::new();

    for entry in read_dir(db)? {
        let entry = entry?;
        if entry.path().is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            packages.push(name.to_string());
        }
    }

    Ok(packages)
}

fn fetch_package(
    index: &HashMap<String, (String, String, String, PathBuf)>,
    package_name: &str,
) -> io::Result<(Package, PathBuf)> {
    let pkg_link = get_link(index, package_name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "package not found"))?;

    let output_path = Path::new("/tmp").join(format!("{package_name}.tar.zst"));

    println!("Downloading {}...", package_name);
    download_file(&pkg_link, &output_path)?;
    download_sig(&pkg_link, &output_path)?;
    let sig = PathBuf::from(format!("{}.sig", output_path.display()));

    verify_sig(&sig, &output_path)?;

    let pkg_meta = read_pkg_info(&output_path).map_err(io::Error::other)?;
    let package = parse_pkg_info(&pkg_meta)?;

    Ok((package, output_path))
}

pub fn update_pkg(
    index: &HashMap<String, (String, String, String, PathBuf)>,
    package_name: &str,
    root: &SomeRoot,
) -> io::Result<()> {
    let (package, archive) = fetch_package(index, package_name)?;

    println!("Unpacking {}...", package_name);

    let files = unpack_package(&archive, &root.root_path).map_err(io::Error::other)?;

    mark_installed(
        package_name,
        &package.version,
        files,
        package.dependencies,
        package.soname_dependencies,
        root,
    )?;
    fs::remove_file(archive)?;

    Ok(())
}

pub fn run_sys_update(root: &SomeRoot) -> io::Result<()> {
    update_mirrors()?;

    let mut index = build_repos_hashmap("core")?;
    let extra = build_repos_hashmap("extra")?;
    index.extend(extra);

    let installed = get_installed_packages(root)?;
    println!("Installed packages loaded: {}", installed.len());

    let outdated = Mutex::new(Vec::new());

    thread::scope(|s| {
        for pkg_name in installed {
            let index = &index;
            let outdated = &outdated;

            s.spawn(move || {
                if let Some((_, _, repo_version, _)) = index.get(&pkg_name) {
                    let local_version = get_installed_version(&pkg_name, root).unwrap();

                    if local_version.trim() != repo_version {
                        outdated.lock().unwrap().push(pkg_name);
                    }
                }
            });
        }
    });

    let outdated = outdated.into_inner().unwrap();

    thread::scope(|s| {
        for pkg in outdated {
            let index = &index;

            s.spawn(move || {
                if let Err(e) = update_pkg(index, &pkg, root) {
                    eprintln!("Failed to update {}: {}", pkg, e);
                }
            });
        }
    });
    Ok(())
}
