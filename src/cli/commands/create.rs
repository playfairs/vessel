use std::{fs, path::Path};
use vessel::{
    VesselError,
    error::io_error,
    image::{self, ImageFormat},
};

pub fn run(image_path: &Path, payload_path: &Path, output: &Path) -> Result<(), VesselError> {
    let image_bytes = fs::read(image_path).map_err(|source| io_error(image_path, source))?;
    let payload = fs::read(payload_path).map_err(|source| io_error(payload_path, source))?;
    let format = image::detect(&image_bytes)?;
    if format != ImageFormat::Png {
        return Err(VesselError::UnsupportedPolyglot(format.name().into()));
    }
    let encoded = vessel::embed(&image_bytes, &payload)?;
    image::validate(format, &encoded)?;
    fs::write(output, encoded).map_err(|source| io_error(output, source))
}
