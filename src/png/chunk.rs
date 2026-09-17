use crate::error::VesselError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PngChunk {
    pub kind: [u8; 4],
    pub data: Vec<u8>,
}

impl PngChunk {
    pub fn new(kind: [u8; 4], data: Vec<u8>) -> Result<Self, VesselError> {
        validate_type(kind)?;
        Ok(Self { kind, data })
    }

    pub fn encode(&self) -> Result<Vec<u8>, VesselError> {
        let length = u32::try_from(self.data.len()).map_err(|_| {
            VesselError::InvalidStructure("chunk payload exceeds u32 length".into())
        })?;
        let mut encoded = Vec::with_capacity(self.data.len() + 12);
        encoded.extend_from_slice(&length.to_be_bytes());
        encoded.extend_from_slice(&self.kind);
        encoded.extend_from_slice(&self.data);
        encoded.extend_from_slice(&crc32(&[&self.kind, &self.data]));
        Ok(encoded)
    }
}

pub fn validate_type(kind: [u8; 4]) -> Result<(), VesselError> {
    if kind.iter().all(|byte| byte.is_ascii_alphabetic()) && kind[2].is_ascii_uppercase() {
        Ok(())
    } else {
        Err(VesselError::InvalidChunkType(kind))
    }
}

pub fn crc32(parts: &[&[u8]]) -> [u8; 4] {
    let mut crc = 0xffff_ffffu32;
    for part in parts {
        for byte in *part {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xedb8_8320
                } else {
                    crc >> 1
                };
            }
        }
    }
    (crc ^ 0xffff_ffff).to_be_bytes()
}
