use std::{fs, path::Path};
use vessel::{VesselError, error::io_error, payload::bundle};

pub fn run(image: &Path, output: &Path) -> Result<(), VesselError> {
    let image_bytes = fs::read(image).map_err(|source| io_error(image, source))?;
    let payload = vessel::extract(&image_bytes)?;
    if bundle::is_bundle(&payload) {
        fs::create_dir_all(output).map_err(|source| io_error(output, source))?;
        bundle::unpack(&payload, output)?;
        Ok(())
    } else {
        fs::write(output, payload).map_err(|source| io_error(output, source))
    }
}
