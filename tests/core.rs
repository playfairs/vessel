use vessel::{
    VesselError, format,
    png::{PngDocument, chunk::crc32},
};

fn minimal_png() -> Vec<u8> {
    let chunks = [
        vessel::png::PngChunk {
            kind: *b"IHDR",
            data: vec![0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0],
        },
        vessel::png::PngChunk {
            kind: *b"IDAT",
            data: vec![120, 156, 99, 0, 0, 0, 2, 0, 1],
        },
        vessel::png::PngChunk {
            kind: *b"IEND",
            data: Vec::new(),
        },
    ];
    let mut bytes = vessel::PNG_SIGNATURE.to_vec();
    for chunk in chunks {
        bytes.extend_from_slice(&(chunk.data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&chunk.kind);
        bytes.extend_from_slice(&chunk.data);
        bytes.extend_from_slice(&crc32(&[&chunk.kind, &chunk.data]));
    }
    bytes
}

#[test]
fn parses_minimal_png_and_round_trips_structure() {
    let input = minimal_png();
    let document = PngDocument::parse(&input).unwrap();
    assert_eq!(document.encode().unwrap(), input);
}

#[test]
fn rejects_invalid_crc() {
    let mut input = minimal_png();
    let last = input.len() - 1;
    input[last] ^= 1;
    assert!(matches!(
        PngDocument::parse(&input),
        Err(VesselError::ChunkCrc { .. })
    ));
}

#[test]
fn embeds_and_extracts_opaque_binary() {
    let payload = [0, 1, 255, 0, 42, 128];
    let embedded = vessel::embed(&minimal_png(), &payload).unwrap();
    assert_eq!(vessel::extract(&embedded).unwrap(), payload);
    let document = PngDocument::parse(&embedded).unwrap();
    assert_eq!(
        document
            .chunks
            .iter()
            .filter(|chunk| chunk.kind == format::VESSEL_TYPE)
            .count(),
        1
    );
}

#[test]
fn rejects_empty_payload() {
    assert!(matches!(
        vessel::embed(&minimal_png(), &[]),
        Err(VesselError::EmptyPayload)
    ));
}

#[test]
fn preserves_existing_chunks_and_multiple_idat_chunks() {
    let mut document = PngDocument::parse(&minimal_png()).unwrap();
    document.chunks.insert(
        1,
        vessel::png::PngChunk {
            kind: *b"tEXt",
            data: b"author\0vessel".to_vec(),
        },
    );
    let idat = document
        .chunks
        .iter()
        .position(|chunk| chunk.kind == *b"IDAT")
        .unwrap();
    document.chunks.insert(
        idat + 1,
        vessel::png::PngChunk {
            kind: *b"IDAT",
            data: vec![1, 2, 3],
        },
    );
    let embedded = vessel::embed(&document.encode().unwrap(), &[9, 8, 7]).unwrap();
    let result = PngDocument::parse(&embedded).unwrap();
    assert_eq!(result.chunks[1].kind, *b"tEXt");
    assert_eq!(result.chunks[1].data, b"author\0vessel");
    assert_eq!(
        result
            .chunks
            .iter()
            .filter(|chunk| chunk.kind == *b"IDAT")
            .count(),
        2
    );
}

#[test]
fn rejects_truncated_png_and_missing_required_chunks() {
    let input = minimal_png();
    assert!(matches!(
        PngDocument::parse(&input[..input.len() - 2]),
        Err(VesselError::ChunkLength { .. }) | Err(VesselError::Truncated { .. })
    ));
    let without_iend = &input[..input.len() - 12];
    assert!(matches!(
        PngDocument::parse(without_iend),
        Err(VesselError::MissingIend)
    ));
    assert!(matches!(
        PngDocument::parse(&input[8..]),
        Err(VesselError::InvalidSignature)
    ));
}

#[test]
fn rejects_corrupt_vessel_payload_and_duplicate_chunks() {
    let embedded = vessel::embed(&minimal_png(), &[1, 2, 3]).unwrap();
    let mut document = PngDocument::parse(&embedded).unwrap();
    let vessel = document
        .chunks
        .iter_mut()
        .find(|chunk| chunk.kind == format::VESSEL_TYPE)
        .unwrap();
    vessel.data[48] ^= 1;
    assert!(matches!(
        vessel::extract(&document.encode().unwrap()),
        Err(VesselError::PayloadDigest)
    ));
    let mut duplicate = PngDocument::parse(&embedded).unwrap();
    let vessel = duplicate
        .chunks
        .iter()
        .find(|chunk| chunk.kind == format::VESSEL_TYPE)
        .unwrap()
        .clone();
    let iend = duplicate
        .chunks
        .iter()
        .position(|chunk| chunk.kind == *b"IEND")
        .unwrap();
    duplicate.chunks.insert(iend, vessel);
    assert!(matches!(
        vessel::extract(&duplicate.encode().unwrap()),
        Err(VesselError::MultipleVesselChunks)
    ));
}
