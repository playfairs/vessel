use std::{fs, path::Path};
use vessel::{
    VesselError,
    error::io_error,
    image::{self, ImageFormat},
    payload::bundle,
};

pub fn run(image_path: &Path, payload_path: &Path, output: &Path) -> Result<(), VesselError> {
    eprintln!("[create] reading image: {}", image_path.display());
    let image_bytes = fs::read(image_path).map_err(|source| io_error(image_path, source))?;
    eprintln!("[create] preparing payload: {}", payload_path.display());
    let mut last_reported = 0;
    let payload = if let Some(bundle) =
        bundle::pack_if_app_executable_with_progress(payload_path, &mut |count, path| {
            if count == 1 || count >= last_reported + 100 {
                eprintln!(
                    "[create] packed {} bundle entries (latest: {})",
                    count,
                    path.display()
                );
                last_reported = count;
            }
        })? {
        eprintln!("[create] bundle payload packed");
        bundle
    } else {
        eprintln!("[create] reading regular payload");
        fs::read(payload_path).map_err(|source| io_error(payload_path, source))?
    };
    eprintln!("[create] validating image format");
    let format = image::detect(&image_bytes)?;
    if format != ImageFormat::Png {
        return Err(VesselError::UnsupportedPolyglot(format.name().into()));
    }
    eprintln!("[create] embedding payload ({} bytes)", payload.len());
    let encoded = vessel::embed(&image_bytes, &payload)?;
    eprintln!("[create] validating generated image");
    image::validate(format, &encoded)?;
    eprintln!("[create] writing output: {}", output.display());
    fs::write(output, encoded).map_err(|source| io_error(output, source))?;
    eprintln!("[create] complete");
    Ok(())
}
