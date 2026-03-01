use anyhow::{Context, Result};
use env_logger;
use log::{self, debug, error, info};
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
        match std::env::consts::OS {
        // match "x86_64-pc-windows-msvc" {
            "linux" => EmulatorBinary {
                source_file: "86box.bin".into(),
                executable: "86box.bin".into(),
                url: "https://ci.86box.net/job/86Box/8453/artifact/New%20Recompiler%20(beta)/Linux%20-%20x64%20(64-bit)/86Box-NDR-Linux-x86_64-b8453.AppImage".into(),
                is_zip_archive: false
            },
            "windows" => EmulatorBinary {
                source_file: "86box.zip".into(),
                executable: "86Box.exe".into(),
                url: "https://ci.86box.net/job/86Box/8453/artifact/New%20Recompiler%20(beta)/Windows%20-%20x64%20(64-bit)/86Box-NDR-Windows-64-b8453.zip".into(),
                is_zip_archive: true
            },
            // "macos" => EmulatorBinary {
            //     source_file: "86box.zip".into(),
            //     executable: "86Box.exe".into(),
            //     url: "https://ci.86box.net/job/86Box/8453/artifact/New%20Recompiler%20(beta)/Windows%20-%20x64%20(64-bit)/86Box-NDR-Windows-64-b8453.zip".into(),
            //     is_zip_archive: true
            // },
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
    debug!("Extract: Open {}", zip_path.as_ref().display());
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(dest_dir)?;
    Ok(())
}

/// Helper to download a file from a URL to a path
fn dl<P: AsRef<Path>>(url: &str, out: P) -> Result<()> {
    let response = ureq::get(url).call()?;
    if response.status() == 200 {
        debug!(
            "Creating {} to receive downloaded data",
            out.as_ref().display()
        );
        let mut file = File::create(&out)?;
        let mut reader = response.into_body().into_reader();
        let bytes_written = copy(&mut reader, &mut file)?;
        debug!(
            "Saved {} bytes to {}",
            bytes_written,
            out.as_ref().display()
        );
    } else {
        error!("Failed to download {:?} {}", response.body(), url);
    }
    Ok(())
}

/// Helper to download a file from a URL and extract to a path
fn dl_extract<P: AsRef<Path>>(url: &str, out: P) -> Result<()> {
    info!("Downloading {url}");
    let temp_name = format!("{}.zip", fastrand::u16(4444..20000));
    dl(url, &temp_name)?;
    extract(&temp_name, out)?;
    std::fs::remove_file(temp_name).context("Can't remove temp file")?;
    Ok(())
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let emulator = EmulatorBinary::new();
    emulator.obtain()?;
    info!("Getting ROMs");
    dl_extract(ROMS_URL, ".")?;
    if Path::new("roms").is_dir() {
        info!("Removing old roms directory");
        std::fs::remove_dir_all("roms")?;
    }
    info!("Renaming roms directory");
    std::fs::rename("roms-5.3", "roms")?;
    info!("Downloading VM");
    dl_extract(VMS_URL, ".")?;
    info!("You can now run ./{} vms/86box.cfg", emulator.executable);
    Ok(())
}
