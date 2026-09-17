use super::chunk::{PngChunk, validate_type};
use crate::{PNG_SIGNATURE, error::VesselError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PngDocument {
    pub chunks: Vec<PngChunk>,
}

impl PngDocument {
    pub fn parse(input: &[u8]) -> Result<Self, VesselError> {
        if input.len() < PNG_SIGNATURE.len() || input[..8] != PNG_SIGNATURE {
            return Err(VesselError::InvalidSignature);
        }
        let mut offset = 8usize;
        let mut chunks = Vec::new();
        let mut saw_ihdr = false;
        let mut saw_iend = false;
        while offset < input.len() {
            if input.len() - offset < 12 {
                return Err(VesselError::Truncated {
                    context: "PNG chunk header",
                });
            }
            let length =
                u32::from_be_bytes(input[offset..offset + 4].try_into().map_err(|_| {
                    VesselError::Truncated {
                        context: "PNG chunk length",
                    }
                })?);
            let kind: [u8; 4] =
                input[offset + 4..offset + 8]
                    .try_into()
                    .map_err(|_| VesselError::Truncated {
                        context: "PNG chunk type",
                    })?;
            validate_type(kind)?;
            let data_start = offset + 8;
            let data_len = usize::try_from(length).map_err(|_| VesselError::ChunkLength {
                kind: String::from_utf8_lossy(&kind).into_owned(),
                length,
            })?;
            let data_end = data_start
                .checked_add(data_len)
                .ok_or(VesselError::Truncated {
                    context: "PNG chunk data",
                })?;
            let crc_end = data_end.checked_add(4).ok_or(VesselError::Truncated {
                context: "PNG chunk CRC",
            })?;
            if crc_end > input.len() {
                return Err(VesselError::ChunkLength {
                    kind: String::from_utf8_lossy(&kind).into_owned(),
                    length,
                });
            }
            let data = input[data_start..data_end].to_vec();
            if input[data_end..crc_end] != super::chunk::crc32(&[&kind, &data]) {
                return Err(VesselError::ChunkCrc {
                    kind: String::from_utf8_lossy(&kind).into_owned(),
                });
            }
            if kind == *b"IHDR" {
                if saw_ihdr || !chunks.is_empty() || data.len() != 13 {
                    return Err(VesselError::InvalidStructure(
                        "IHDR must be the first 13-byte chunk".into(),
                    ));
                }
                saw_ihdr = true;
            } else if !saw_ihdr {
                return Err(VesselError::MissingIhdr);
            }
            if kind == *b"IEND" {
                if !data.is_empty() {
                    return Err(VesselError::InvalidStructure(
                        "IEND must have an empty payload".into(),
                    ));
                }
                saw_iend = true;
                chunks.push(PngChunk { kind, data });
                offset = crc_end;
                break;
            }
            chunks.push(PngChunk { kind, data });
            offset = crc_end;
        }
        if !saw_ihdr {
            return Err(VesselError::MissingIhdr);
        }
        if !saw_iend {
            return Err(VesselError::MissingIend);
        }
        if offset != input.len() {
            return Err(VesselError::DataAfterIend);
        }
        if !chunks.iter().any(|chunk| chunk.kind == *b"IDAT") {
            return Err(VesselError::InvalidStructure(
                "PNG must contain at least one IDAT chunk".into(),
            ));
        }
        Ok(Self { chunks })
    }

    pub fn encode(&self) -> Result<Vec<u8>, VesselError> {
        let mut output = PNG_SIGNATURE.to_vec();
        for chunk in &self.chunks {
            output.extend_from_slice(&chunk.encode()?);
        }
        Ok(output)
    }
}
