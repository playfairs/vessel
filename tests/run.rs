use std::{fs, process::Command};

#[test]
fn create_and_run_forward_payload_arguments() {
    let directory = tempfile::tempdir().unwrap();
    let image = directory.path().join("image.png");
    let output = directory.path().join("renamed.data");
    let hello = "examples/binaries/hello";
    fs::copy("examples/images/png/image.png", &image).unwrap();
    let binary = env!("CARGO_BIN_EXE_vessel");
    assert!(
        Command::new(binary)
            .args([
                "create",
                image.to_str().unwrap(),
                hello,
                "-o",
                output.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    let status = Command::new(binary)
        .args(["run", output.to_str().unwrap(), "alpha", "beta"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(
        Command::new(binary)
            .args(["verify", output.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn create_rejects_non_png_polyglot_claims() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("image.out");
    let binary = env!("CARGO_BIN_EXE_vessel");
    let status = Command::new(binary)
        .args([
            "create",
            "examples/images/jpeg/image.jpeg",
            "examples/binaries/hello",
            "-o",
            output.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(!status.success());
}

#[test]
fn create_and_run_preserves_macos_app_bundle_layout() {
    let directory = tempfile::tempdir().unwrap();
    let app = directory.path().join("STRAFTAT.app");
    let executable_directory = app.join("Contents/MacOS");
    fs::create_dir_all(&executable_directory).unwrap();
    fs::copy(
        "examples/binaries/hello",
        executable_directory.join("STRAFTAT"),
    )
    .unwrap();
    let image = directory.path().join("image.png");
    let output = directory.path().join("vessel.png");
    let extracted = directory.path().join("extracted.app");
    fs::copy("examples/images/png/image.png", &image).unwrap();
    let binary = env!("CARGO_BIN_EXE_vessel");

    assert!(
        Command::new(binary)
            .args([
                "create",
                image.to_str().unwrap(),
                app.join("Contents/MacOS/STRAFTAT").to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        Command::new(binary)
            .args(["run", output.to_str().unwrap(), "alpha", "beta"])
            .status()
            .unwrap()
            .code(),
        Some(0)
    );
    assert!(
        Command::new(binary)
            .args([
                "extract",
                output.to_str().unwrap(),
                "-o",
                extracted.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success()
    );
    assert!(extracted.join("Contents/MacOS/STRAFTAT").is_file());
}
