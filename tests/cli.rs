use std::{fs, process::Command};

fn png() -> Vec<u8> {
    let chunk = |kind: &[u8; 4], data: &[u8]| {
        let mut bytes = (data.len() as u32).to_be_bytes().to_vec();
        bytes.extend_from_slice(kind);
        bytes.extend_from_slice(data);
        bytes.extend_from_slice(&vessel::png::chunk::crc32(&[kind, data]));
        bytes
    };
    let mut bytes = vessel::PNG_SIGNATURE.to_vec();
    bytes.extend(chunk(b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0]));
    bytes.extend(chunk(b"IDAT", &[1, 2, 3]));
    bytes.extend(chunk(b"IEND", &[]));
    bytes
}

#[test]
fn cli_embeds_extracts_and_verifies_files() {
    let directory = tempfile::tempdir().unwrap();
    let image = directory.path().join("image.png");
    let payload = directory.path().join("payload.bin");
    let output = directory.path().join("vessel.png");
    let extracted = directory.path().join("extracted.bin");
    fs::write(&image, png()).unwrap();
    fs::write(&payload, [0, 255, 1, 0]).unwrap();
    let binary = env!("CARGO_BIN_EXE_vessel");
    assert!(
        Command::new(binary)
            .args([
                "embed",
                image.to_str().unwrap(),
                payload.to_str().unwrap(),
                "-o",
                output.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["verify", output.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(binary)
            .args([
                "extract",
                output.to_str().unwrap(),
                "-o",
                extracted.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(fs::read(payload).unwrap(), fs::read(extracted).unwrap());
}
