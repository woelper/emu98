use anyhow::Result;
use env_logger;
use log::{self, debug, info};
use std::fs::File;
use std::io::copy;
use std::path::Path;
use zip::ZipArchive;
static ROMS_URL: &str = "https://github.com/86Box/roms/archive/refs/tags/v5.3.zip";
static VMS_URL: &str = "https://huggingface.co/datasets/johnsor/win98/resolve/main/win98_image.zip";

struct EmulatorBinary {
    /// Local artifact name
    source_file: String,
    /// Local executable name
    executable: String,
    /// URL to download the artifact from
    url: String,
    is_zip_archive: bool,
}

impl EmulatorBinary {
    fn new() -> Self {
        match current_platform::CURRENT_PLATFORM {
        // match "x86_64-pc-windows-msvc" {
            "x86_64-unknown-linux-gnu" => EmulatorBinary {
                source_file: "86box.bin".into(),
                executable: "86box.bin".into(),
                url: "https://ci.86box.net/job/86Box/8453/artifact/New%20Recompiler%20(beta)/Linux%20-%20x64%20(64-bit)/86Box-NDR-Linux-x86_64-b8453.AppImage".into(),
                is_zip_archive: false
            },
            "x86_64-pc-windows-msvc" => EmulatorBinary {
                source_file: "86box.zip".into(),
                executable: "86Box.exe".into(),
                url: "https://ci.86box.net/job/86Box/8453/artifact/New%20Recompiler%20(beta)/Windows%20-%20x64%20(64-bit)/86Box-NDR-Windows-64-b8453.zip".into(),
                is_zip_archive: true
            },
            _ => panic!("Unsupported platform"),
        }
    }

    fn obtain(&self) -> Result<()> {
        info!("Downloading emulator");
        dl(&self.url, Path::new(&self.source_file))?;
        if self.is_zip_archive {
            info!("Extracting emulator");
            extract(&self.source_file, ".")?;
            std::fs::remove_file(&self.source_file)?;
        }
        Ok(())
    }
}

fn extract<P: AsRef<Path>, Q: AsRef<Path>>(zip_path: P, dest_dir: Q) -> Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(dest_dir)?;
    Ok(())
}

/// Helper to download a file from a URL to a path
fn dl<P: AsRef<Path>>(url: &str, out: P) -> Result<()> {
    let response = ureq::get(url).call()?;
    if response.status() == 200 {
        let mut file = File::create(out)?;
        let mut reader = response.into_body().into_reader();
        let bytes_written = copy(&mut reader, &mut file)?;
        debug!("Saved {} bytes.", bytes_written);
    }
    Ok(())
}

/// Helper to download a file from a URL and extract to a path
fn dl_extract<P: AsRef<Path>>(url: &str, out: P) -> Result<()> {
    info!("Downloading {url}");
    dl(url, "tmp.zip")?;
    info!("Extracting");
    extract("tmp.zip", out)?;
    std::fs::remove_file("tmp.zip")?;
    Ok(())
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    let emulator = EmulatorBinary::new();
    emulator.obtain()?;
    dl_extract(ROMS_URL, ".")?;
    info!("Removing old roms directory");
    std::fs::remove_dir_all("roms")?;
    std::fs::rename("roms-5.3", "roms")?;
    info!("Downloading VM");
    dl_extract(VMS_URL, ".")?;
    info!("You can now run ./{} vms/86box.cfg", emulator.executable);
    Ok(())
}
