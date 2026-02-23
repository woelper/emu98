use anyhow::Result;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use zip::ZipArchive;
static ROMS_URL: &str = "https://github.com/86Box/roms/archive/refs/tags/v5.3.zip";

fn get_86box_url() -> String {
    "https://ci.86box.net/job/86Box/8436/artifact/New%20Recompiler%20(beta)/Linux%20-%20x64%20(64-bit)/86Box-NDR-Linux-x86_64-b8436.AppImage".into()
}

/// Helper to download a file from a URL to a path
fn dl(url: &str, out: &Path) -> Result<()> {
    let response = ureq::get(url).call()?;
    if response.status() == 200 {
        let mut file = File::create(out)?;
        let mut reader = response.into_body().into_reader();
        let bytes_written = copy(&mut reader, &mut file)?;
        println!("Saved {} bytes.", bytes_written);
    }
    Ok(())
}

fn main() -> Result<()> {
    println!("Downloading emulator");
    dl(&get_86box_url(), Path::new("out.bin"))?;
    println!("DL roms");
    dl(ROMS_URL, Path::new("roms.zip"))?;
    let file = File::open("roms.zip")?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(".")?;
    std::fs::rename("roms-5.3", "roms")?;
    Ok(())
}
