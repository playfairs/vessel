use std::fs;
use vessel::image::{self, ImageFormat};

#[test]
fn detects_and_validates_the_real_example_corpus() {
    let cases = [
        ("examples/images/png/image.png", ImageFormat::Png),
        ("examples/images/jpeg/image.jpeg", ImageFormat::Jpeg),
        ("examples/images/jpg/image.jpg", ImageFormat::Jpeg),
        ("examples/images/webp/image.webp", ImageFormat::Webp),
        ("examples/images/gif/image.gif", ImageFormat::Gif),
        ("examples/images/bmp/image.bmp", ImageFormat::Bmp),
        ("examples/images/pbm/image.pbm", ImageFormat::Pbm),
        ("examples/images/pgm/image.pgm", ImageFormat::Pgm),
        ("examples/images/ppm/image.ppm", ImageFormat::Ppm),
        ("examples/images/tga/image.tga", ImageFormat::Tga),
        ("examples/images/tiff/image.tiff", ImageFormat::Tiff),
    ];
    for (path, expected) in cases {
        let bytes = fs::read(path).unwrap();
        assert_eq!(image::detect(&bytes).unwrap(), expected, "{path}");
    }
}

#[test]
fn rejects_unknown_and_truncated_images() {
    assert!(image::detect(b"not an image").is_err());
    assert!(image::detect(&fs::read("examples/images/jpeg/image.jpeg").unwrap()[..3]).is_err());
    assert!(image::detect(&fs::read("examples/images/webp/image.webp").unwrap()[..12]).is_err());
}
