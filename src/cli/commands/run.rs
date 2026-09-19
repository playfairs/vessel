use std::{fs, path::Path, process::Command};
use vessel::{VesselError, error::io_error, image, payload::bundle};

pub fn run(image_path: &Path, arguments: &[String]) -> Result<i32, VesselError> {
    eprintln!("[run] reading image: {}", image_path.display());
    let image_bytes = fs::read(image_path).map_err(|source| io_error(image_path, source))?;
    eprintln!("[run] validating image format");
    let format = image::detect(&image_bytes)?;
    if format != image::ImageFormat::Png {
        return Err(VesselError::UnsupportedPolyglot(format.name().into()));
    }
    eprintln!("[run] extracting and verifying payload");
    let payload = vessel::extract(&image_bytes)?;
    let directory =
        tempfile::tempdir().map_err(|source| io_error(Path::new("temporary directory"), source))?;
    let executable = if bundle::is_bundle(&payload) {
        eprintln!("[run] reconstructing application bundle");
        bundle::unpack(&payload, directory.path())?
    } else {
        eprintln!("[run] materializing regular executable");
        let executable = directory.path().join("payload");
        fs::write(&executable, payload).map_err(|source| io_error(&executable, source))?;
        set_executable(&executable)?;
        executable
    };
    eprintln!("[run] launching: {}", executable.display());
    let status = Command::new(&executable)
        .args(arguments)
        .status()
        .map_err(|source| io_error(&executable, source))?;
    Ok(status.code().unwrap_or(128 + status.signal().unwrap_or(0)))
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), VesselError> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|source| io_error(path, source))?
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).map_err(|source| io_error(path, source))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), VesselError> {
    Ok(())
}

trait ExitSignal {
    fn signal(&self) -> Option<i32>;
}
impl ExitSignal for std::process::ExitStatus {
    #[cfg(unix)]
    fn signal(&self) -> Option<i32> {
        std::os::unix::process::ExitStatusExt::signal(self)
    }
    #[cfg(not(unix))]
    fn signal(&self) -> Option<i32> {
        None
    }
}
