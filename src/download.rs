use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::thread;

use crate::SomeRoot;
use crate::commands::verify_sig::verify_sig;
use crate::repo::get_link;

pub fn download_file(url: &str, output_path: &Path) -> io::Result<()> {
    let path_str = output_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "incorrect file path"))?;

    let status = Command::new("curl")
        .args(["-fLs", "-o", path_str, url])
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

pub fn run_new_install(
    index: Arc<HashMap<String, (String, String, String, PathBuf)>>,
    packages: HashSet<String>,
    root: &SomeRoot,
) -> io::Result<HashMap<String, PathBuf>> {
    let cache_dir = root.root_path.join("var/cache/rpk/pkgs");
    std::fs::create_dir_all(&cache_dir)?;
    println!("{:?}", &cache_dir);
    std::fs::create_dir_all(&cache_dir)?;

    let handles: Vec<_> = packages
        .into_iter()
        .map(|pkg| {
            let index = Arc::clone(&index);
            let cache_dir = cache_dir.clone();

            thread::spawn(move || -> io::Result<(String, PathBuf)> {
                let link = get_link(&index, &pkg)
                    .ok_or_else(|| io::Error::other(format!("package {pkg} not found")))?;

                let path = cache_dir.join(format!("{pkg}.pkg.tar.zst"));

                if !path.exists() {
                    println!("Downloading {pkg}");

                    let sig_url = format!("{link}.sig");
                    let sig = PathBuf::from(format!("{}.sig", path.display()));
                    let tmp = path.with_extension("pkg.tar.zst.part");

                    download_file(&sig_url, &sig)?;
                    download_file(&link, &tmp)?;

                    if let Err(e) = verify_sig(&sig, &tmp) {
                        let _ = std::fs::remove_file(&tmp);
                        let _ = std::fs::remove_file(&sig);
                        return Err(e);
                    }

                    std::fs::rename(&tmp, &path)?;
                    std::fs::remove_file(&sig)?;

                    println!("Downloaded {pkg}");
                }

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
