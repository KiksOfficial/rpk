use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::thread;

use crate::commands::verify_sig::verify_sig;
use crate::repo::get_link;

pub fn download_file(url: &str, output_path: &Path) -> io::Result<()> {
    let path_str = output_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "incorrect file path"))?;

    let status = Command::new("curl")
        .args(["-fL", "-o", path_str, url])
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
) -> io::Result<HashMap<String, PathBuf>> {
    let handles: Vec<_> = packages
        .into_iter()
        .map(|pkg| {
            let index = Arc::clone(&index);

            thread::spawn(move || -> io::Result<(String, PathBuf)> {
                let link = get_link(&index, &pkg)
                    .ok_or_else(|| io::Error::other(format!("package {pkg} not found")))?;

                std::fs::create_dir_all("/var/cache/rpk/pkgs")?;
                let path = PathBuf::from(format!("/var/cache/rpk/pkgs/{pkg}.pkg.tar.zst"));

                if !path.exists() {
                    println!("Downloading {pkg}");

                    let sig_url = format!("{}.sig", link);
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

                    println!("Downloading {}", &pkg);
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
