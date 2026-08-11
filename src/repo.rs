use std::{
    collections::{HashMap, HashSet},
    fs::{read_dir, read_to_string},
    io::{self},
    path::{Path, PathBuf},
};

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
