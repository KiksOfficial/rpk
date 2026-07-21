use std::fs;
use std::io;

pub fn list_installed() -> io::Result<()> {
    let faili_sisu = fs::read_to_string("/home/kiks/Proge/fake-root/var/lib/rpk_db.txt")?;
    for rida in faili_sisu.lines() {
        if let Some(name) = rida.split_once(':') {
            println!("{}", &name.0.trim());
        }
    }
    Ok(())
}
