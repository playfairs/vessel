use std::{fs, path::Path};
use vessel::{VesselError, error::io_error, payload::bundle};

pub fn run(image: &Path, payload: &Path, output: &Path) -> Result<(), VesselError> {
    let image_bytes = fs::read(image).map_err(|source| io_error(image, source))?;
    let payload_bytes = if let Some(bundle) = bundle::pack_if_app_executable(payload)? {
        bundle
    } else {
        fs::read(payload).map_err(|source| io_error(payload, source))?
    };
    let encoded = vessel::embed(&image_bytes, &payload_bytes)?;
    fs::write(output, encoded).map_err(|source| io_error(output, source))
}
