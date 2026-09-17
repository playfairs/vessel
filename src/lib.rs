pub mod error;
pub mod format;
pub mod payload;
pub mod png;

pub use error::VesselError;

pub const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

pub fn embed(image: &[u8], payload: &[u8]) -> Result<Vec<u8>, VesselError> {
    payload::embedder::embed(image, payload)
}

pub fn extract(image: &[u8]) -> Result<Vec<u8>, VesselError> {
    payload::extractor::extract(image)
}
