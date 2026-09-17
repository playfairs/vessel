use thiserror::Error;

#[derive(Debug, Error)]
pub enum VesselError {
    #[error("filesystem access failed for {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("PNG signature is invalid")]
    InvalidSignature,
    #[error("PNG is truncated while reading {context}")]
    Truncated { context: &'static str },
    #[error("PNG chunk type is invalid: {0:?}")]
    InvalidChunkType([u8; 4]),
    #[error("PNG chunk {kind} has length {length} exceeding the input")]
    ChunkLength { kind: String, length: u32 },
    #[error("PNG chunk {kind} has an invalid CRC")]
    ChunkCrc { kind: String },
    #[error("PNG is missing IHDR")]
    MissingIhdr,
    #[error("PNG is missing IEND")]
    MissingIend,
    #[error("PNG contains data after IEND")]
    DataAfterIend,
    #[error("PNG contains an invalid chunk order: {0}")]
    InvalidStructure(String),
    #[error("Vessel payload cannot be empty")]
    EmptyPayload,
    #[error("Vessel chunk is malformed: {0}")]
    MalformedVessel(String),
    #[error("Vessel format version {0} is unsupported")]
    UnsupportedVersion(u16),
    #[error("Vessel payload length is invalid: declared {declared}, available {available}")]
    PayloadLength { declared: u64, available: usize },
    #[error("Vessel payload digest does not match")]
    PayloadDigest,
    #[error("PNG contains multiple Vessel chunks")]
    MultipleVesselChunks,
}

pub fn io_error(path: &std::path::Path, source: std::io::Error) -> VesselError {
    VesselError::Io {
        path: path.display().to_string(),
        source,
    }
}
