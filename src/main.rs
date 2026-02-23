use anyhow::Result;
use env_logger;
use log::{self, debug, info};
use std::fs::File;
use std::io::copy;
use std::path::Path;
use zip::ZipArchive;
static ROMS_URL: &str = "https://github.com/86Box/roms/archive/refs/tags/v5.3.zip";

fn get_86box_url() -> String {
    "https://ci.86box.net/job/86Box/8436/artifact/New%20Recompiler%20(beta)/Linux%20-%20x64%20(64-bit)/86Box-NDR-Linux-x86_64-b8436.AppImage".into()
}

struct EmulatorBinary {
    /// Final artifact name
    filename: String,
    /// URL to download the artifact from
    url: String,
    is_zip_archive: bool,
}

impl EmulatorBinary {
    fn new() -> Self {
        match current_platform::CURRENT_PLATFORM {
            "x86_64-unknown-linux-gnu" => EmulatorBinary {
                filename: "86box.bin".into(),
                url: "https://ci.86box.net/job/86Box/8453/artifact/New%20Recompiler%20(beta)/Linux%20-%20x64%20(64-bit)/86Box-NDR-Linux-x86_64-b8453.AppImage".into(),
                is_zip_archive: false
            },
            _ => panic!("Unsupported platform"),
        }
    }
}

/// Helper to download a file from a URL to a path
fn dl(url: &str, out: &Path) -> Result<()> {
    let response = ureq::get(url).call()?;
    if response.status() == 200 {
        let mut file = File::create(out)?;
        let mut reader = response.into_body().into_reader();
        let bytes_written = copy(&mut reader, &mut file)?;
        debug!("Saved {} bytes.", bytes_written);
    }
    Ok(())
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    let emulator = EmulatorBinary::new();
    info!("Downloading emulator");
    dl(&emulator.url, Path::new(&emulator.filename))?;
    info!("Downloading ROMs");
    dl(ROMS_URL, Path::new("roms.zip"))?;
    info!("Extracting ROMs");
    let file = File::open("roms.zip")?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(".")?;
    info!("Removing old roms directory");
    std::fs::remove_dir_all("roms")?;
    std::fs::rename("roms-5.3", "roms")?;
    std::fs::remove_file("roms.zip")?;
    info!("You can now run {}", emulator.filename);
    Ok(())
}
