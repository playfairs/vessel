use std::{fs, path::Path};
use vessel::{VesselError, error::io_error};

pub fn run(image: &Path, output: &Path) -> Result<(), VesselError> {
    let image_bytes = fs::read(image).map_err(|source| io_error(image, source))?;
    let payload = vessel::extract(&image_bytes)?;
    fs::write(output, payload).map_err(|source| io_error(output, source))
}
