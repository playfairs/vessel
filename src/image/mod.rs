use crate::{error::VesselError, png};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
    Gif,
    Bmp,
    Pbm,
    Pgm,
    Ppm,
    Tga,
    Tiff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    ExecutablePolyglot,
    ContainerOnly,
    Unsupported,
}

impl ImageFormat {
    pub fn name(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Webp => "WebP",
            Self::Gif => "GIF",
            Self::Bmp => "BMP",
            Self::Pbm => "PBM",
            Self::Pgm => "PGM",
            Self::Ppm => "PPM",
            Self::Tga => "TGA",
            Self::Tiff => "TIFF",
        }
    }

    pub fn capability(self) -> Capability {
        match self {
            Self::Png => Capability::ContainerOnly,
            Self::Jpeg
            | Self::Webp
            | Self::Gif
            | Self::Bmp
            | Self::Pbm
            | Self::Pgm
            | Self::Ppm
            | Self::Tga
            | Self::Tiff => Capability::Unsupported,
        }
    }
}

pub fn detect(input: &[u8]) -> Result<ImageFormat, VesselError> {
    let format = if input.starts_with(&crate::PNG_SIGNATURE) {
        ImageFormat::Png
    } else if input.starts_with(&[0xff, 0xd8, 0xff]) {
        ImageFormat::Jpeg
    } else if input.len() >= 12 && &input[..4] == b"RIFF" && &input[8..12] == b"WEBP" {
        ImageFormat::Webp
    } else if input.starts_with(b"GIF87a") || input.starts_with(b"GIF89a") {
        ImageFormat::Gif
    } else if input.starts_with(b"BM") {
        ImageFormat::Bmp
    } else if input.starts_with(b"P1") || input.starts_with(b"P4") {
        ImageFormat::Pbm
    } else if input.starts_with(b"P2") || input.starts_with(b"P5") {
        ImageFormat::Pgm
    } else if input.starts_with(b"P3") || input.starts_with(b"P6") {
        ImageFormat::Ppm
    } else if input.len() >= 4
        && ((&input[..2] == b"II" || &input[..2] == b"MM") && input[2..4] == [42, 0]
            || input[2..4] == [0, 42])
    {
        ImageFormat::Tiff
    } else if looks_like_tga(input) {
        ImageFormat::Tga
    } else {
        return Err(VesselError::UnsupportedImage(
            "unknown binary signature".into(),
        ));
    };
    validate(format, input)?;
    Ok(format)
}

pub fn validate(format: ImageFormat, input: &[u8]) -> Result<(), VesselError> {
    match format {
        ImageFormat::Png => {
            png::validator::validate(input)?;
        }
        ImageFormat::Jpeg => validate_jpeg(input)?,
        ImageFormat::Webp => validate_webp(input)?,
        ImageFormat::Gif => validate_gif(input)?,
        ImageFormat::Bmp => validate_bmp(input)?,
        ImageFormat::Pbm | ImageFormat::Pgm | ImageFormat::Ppm => validate_netpbm(input, format)?,
        ImageFormat::Tga => validate_tga(input)?,
        ImageFormat::Tiff => validate_tiff(input)?,
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> VesselError {
    VesselError::InvalidImage(message.into())
}

fn read_be_u16(input: &[u8], offset: usize) -> Result<u16, VesselError> {
    input
        .get(offset..offset + 2)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u16::from_be_bytes)
        .ok_or_else(|| invalid("truncated integer"))
}

fn read_le_u16(input: &[u8], offset: usize) -> Result<u16, VesselError> {
    input
        .get(offset..offset + 2)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| invalid("truncated integer"))
}

fn read_le_u32(input: &[u8], offset: usize) -> Result<u32, VesselError> {
    input
        .get(offset..offset + 4)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| invalid("truncated integer"))
}

fn read_be_u32(input: &[u8], offset: usize) -> Result<u32, VesselError> {
    input
        .get(offset..offset + 4)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u32::from_be_bytes)
        .ok_or_else(|| invalid("truncated integer"))
}

fn validate_jpeg(input: &[u8]) -> Result<(), VesselError> {
    if input.len() < 4 || input[..2] != [0xff, 0xd8] {
        return Err(invalid("JPEG SOI is missing"));
    }
    let mut offset = 2usize;
    let mut saw_eoi = false;
    while offset < input.len() {
        if input[offset] != 0xff {
            return Err(invalid("JPEG marker prefix is missing"));
        }
        while offset < input.len() && input[offset] == 0xff {
            offset += 1;
        }
        let marker = *input
            .get(offset)
            .ok_or_else(|| invalid("JPEG marker is truncated"))?;
        offset += 1;
        if marker == 0xd9 {
            saw_eoi = true;
            break;
        }
        if marker == 0xda {
            let length = usize::from(read_be_u16(input, offset)?);
            if length < 2 || offset.checked_add(length).is_none() || offset + length > input.len() {
                return Err(invalid("JPEG scan header is truncated"));
            }
            offset += length;
            while offset + 1 < input.len() {
                if input[offset] == 0xff && input[offset + 1] == 0xd9 {
                    saw_eoi = true;
                    break;
                }
                offset += 1;
            }
            break;
        }
        if (0xd0..=0xd7).contains(&marker) || marker == 0x01 {
            continue;
        }
        let length = usize::from(read_be_u16(input, offset)?);
        if length < 2 || offset.checked_add(length).is_none() || offset + length > input.len() {
            return Err(invalid("JPEG segment is truncated"));
        }
        offset += length;
    }
    if saw_eoi {
        Ok(())
    } else {
        Err(invalid("JPEG EOI is missing"))
    }
}

fn validate_webp(input: &[u8]) -> Result<(), VesselError> {
    if input.len() < 12 || &input[..4] != b"RIFF" || &input[8..12] != b"WEBP" {
        return Err(invalid("WebP RIFF header is invalid"));
    }
    let declared =
        usize::try_from(read_le_u32(input, 4)?).map_err(|_| invalid("WebP RIFF size overflows"))?;
    if declared.checked_add(8) != Some(input.len()) {
        return Err(invalid("WebP RIFF size does not match input"));
    }
    let mut offset = 12usize;
    while offset < input.len() {
        if input.len() - offset < 8 {
            return Err(invalid("WebP chunk header is truncated"));
        }
        let length = usize::try_from(read_le_u32(input, offset + 4)?)
            .map_err(|_| invalid("WebP chunk size overflows"))?;
        let padded = length
            .checked_add(length & 1)
            .ok_or_else(|| invalid("WebP chunk size overflows"))?;
        offset = offset
            .checked_add(8)
            .and_then(|value| value.checked_add(padded))
            .ok_or_else(|| invalid("WebP chunk offset overflows"))?;
        if offset > input.len() {
            return Err(invalid("WebP chunk exceeds input"));
        }
    }
    Ok(())
}

fn validate_gif(input: &[u8]) -> Result<(), VesselError> {
    if input.len() < 13 || !(input.starts_with(b"GIF87a") || input.starts_with(b"GIF89a")) {
        return Err(invalid("GIF header is invalid"));
    }
    let width = usize::from(read_le_u16(input, 6)?);
    let height = usize::from(read_le_u16(input, 8)?);
    if width == 0 || height == 0 {
        return Err(invalid("GIF dimensions are zero"));
    }
    let packed = input[10];
    let mut offset = 13usize;
    if packed & 0x80 != 0 {
        offset = offset
            .checked_add(
                3usize
                    .checked_mul(1usize << (usize::from(packed & 7) + 1))
                    .ok_or_else(|| invalid("GIF color table overflows"))?,
            )
            .ok_or_else(|| invalid("GIF color table overflows"))?;
    }
    while offset < input.len() {
        match input[offset] {
            0x3b => return Ok(()),
            0x21 => {
                offset += 2;
                while offset < input.len() {
                    let size = usize::from(input[offset]);
                    offset += 1;
                    if size == 0 {
                        break;
                    }
                    offset = offset
                        .checked_add(size)
                        .ok_or_else(|| invalid("GIF extension overflows"))?;
                }
            }
            0x2c => {
                if offset + 10 > input.len() {
                    return Err(invalid("GIF image descriptor is truncated"));
                }
                let descriptor = input[offset + 9];
                offset += 10;
                if descriptor & 0x80 != 0 {
                    offset = offset
                        .checked_add(
                            3usize
                                .checked_mul(1usize << (usize::from(descriptor & 7) + 1))
                                .ok_or_else(|| invalid("GIF color table overflows"))?,
                        )
                        .ok_or_else(|| invalid("GIF color table overflows"))?;
                }
                if offset >= input.len() {
                    return Err(invalid("GIF image data is truncated"));
                }
                offset += 1;
                while offset < input.len() {
                    let size = usize::from(input[offset]);
                    offset += 1;
                    if size == 0 {
                        break;
                    }
                    offset = offset
                        .checked_add(size)
                        .ok_or_else(|| invalid("GIF image data overflows"))?;
                }
            }
            _ => return Err(invalid("GIF block introducer is invalid")),
        }
        if offset > input.len() {
            return Err(invalid("GIF block exceeds input"));
        }
    }
    Err(invalid("GIF trailer is missing"))
}

fn validate_bmp(input: &[u8]) -> Result<(), VesselError> {
    if input.len() < 54 || &input[..2] != b"BM" {
        return Err(invalid("BMP header is invalid"));
    }
    let size =
        usize::try_from(read_le_u32(input, 2)?).map_err(|_| invalid("BMP size overflows"))?;
    let pixels = usize::try_from(read_le_u32(input, 10)?)
        .map_err(|_| invalid("BMP pixel offset overflows"))?;
    let dib =
        usize::try_from(read_le_u32(input, 14)?).map_err(|_| invalid("BMP DIB size overflows"))?;
    if size != input.len()
        || dib < 12
        || 14usize.checked_add(dib).is_none_or(|end| end > input.len())
        || pixels > input.len()
    {
        return Err(invalid("BMP bounds are invalid"));
    }
    Ok(())
}

fn netpbm_tokens(input: &[u8]) -> Vec<&[u8]> {
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < input.len() {
        while index < input.len() && (input[index].is_ascii_whitespace() || input[index] == b'#') {
            if input[index] == b'#' {
                while index < input.len() && input[index] != b'\n' {
                    index += 1;
                }
            } else {
                index += 1;
            }
        }
        let start = index;
        while index < input.len() && !input[index].is_ascii_whitespace() && input[index] != b'#' {
            index += 1;
        }
        if start < index {
            tokens.push(&input[start..index]);
        }
    }
    tokens
}

fn validate_netpbm(input: &[u8], format: ImageFormat) -> Result<(), VesselError> {
    let tokens = netpbm_tokens(input);
    if tokens.len() < 3 {
        return Err(invalid("Netpbm header is truncated"));
    }
    let magic = tokens[0];
    let expected = match format {
        ImageFormat::Pbm => [b"P1".as_slice(), b"P4".as_slice()],
        ImageFormat::Pgm => [b"P2".as_slice(), b"P5".as_slice()],
        _ => [b"P3".as_slice(), b"P6".as_slice()],
    };
    if !expected.contains(&magic) {
        return Err(invalid("Netpbm magic does not match format"));
    }
    let width = std::str::from_utf8(tokens[1])
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| invalid("Netpbm width is invalid"))?;
    let height = std::str::from_utf8(tokens[2])
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| invalid("Netpbm height is invalid"))?;
    if width == 0 || height == 0 {
        return Err(invalid("Netpbm dimensions are zero"));
    }
    if format != ImageFormat::Pbm && tokens.len() < 4 {
        return Err(invalid("Netpbm maximum value is missing"));
    }
    Ok(())
}

fn validate_tga(input: &[u8]) -> Result<(), VesselError> {
    if input.len() < 18 {
        return Err(invalid("TGA header is truncated"));
    }
    let width = usize::from(read_le_u16(input, 12)?);
    let height = usize::from(read_le_u16(input, 14)?);
    let depth = input[16];
    if width == 0 || height == 0 || ![8, 15, 16, 24, 32].contains(&depth) {
        return Err(invalid("TGA dimensions or pixel depth is invalid"));
    }
    let image_type = input[2];
    if ![1, 2, 3, 9, 10, 11].contains(&image_type) {
        return Err(invalid("TGA image type is unsupported"));
    }
    let bytes_per_pixel = usize::from(depth).div_ceil(8);
    let color_map_bytes = if input[1] == 0 {
        0
    } else {
        let entries = usize::from(read_le_u16(input, 5)?);
        let entry_bytes = usize::from(input[7]).div_ceil(8);
        entries
            .checked_mul(entry_bytes)
            .ok_or_else(|| invalid("TGA color map size overflows"))?
    };
    let offset = 18usize
        .checked_add(usize::from(input[0]))
        .and_then(|value| value.checked_add(color_map_bytes))
        .ok_or_else(|| invalid("TGA offset overflows"))?;
    if offset >= input.len() {
        return Err(invalid("TGA image data is truncated"));
    }
    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| invalid("TGA pixel count overflows"))?;
    if [9, 10, 11].contains(&image_type) {
        let mut cursor = offset;
        let mut decoded = 0usize;
        while decoded < pixel_count {
            let packet = *input
                .get(cursor)
                .ok_or_else(|| invalid("TGA RLE packet is truncated"))?;
            cursor += 1;
            let count = usize::from(packet & 0x7f) + 1;
            if count > pixel_count - decoded {
                return Err(invalid("TGA RLE packet exceeds image dimensions"));
            }
            let packet_bytes = if packet & 0x80 != 0 {
                bytes_per_pixel
            } else {
                count
                    .checked_mul(bytes_per_pixel)
                    .ok_or_else(|| invalid("TGA RLE packet size overflows"))?
            };
            cursor = cursor
                .checked_add(packet_bytes)
                .ok_or_else(|| invalid("TGA RLE offset overflows"))?;
            if cursor > input.len() {
                return Err(invalid("TGA RLE data is truncated"));
            }
            decoded += count;
        }
    } else {
        let pixels = pixel_count
            .checked_mul(bytes_per_pixel)
            .ok_or_else(|| invalid("TGA pixel size overflows"))?;
        if pixels > input.len() - offset {
            return Err(invalid("TGA image data is truncated"));
        }
    }
    Ok(())
}

fn validate_tiff(input: &[u8]) -> Result<(), VesselError> {
    if input.len() < 8
        || !((&input[..2] == b"II" && input[2..4] == [42, 0])
            || (&input[..2] == b"MM" && input[2..4] == [0, 42]))
    {
        return Err(invalid("TIFF header is invalid"));
    }
    let little = &input[..2] == b"II";
    let offset = if little {
        usize::try_from(read_le_u32(input, 4)?).map_err(|_| invalid("TIFF IFD offset overflows"))?
    } else {
        usize::try_from(read_be_u32(input, 4)?).map_err(|_| invalid("TIFF IFD offset overflows"))?
    };
    if offset + 2 > input.len() {
        return Err(invalid("TIFF IFD is out of bounds"));
    }
    let count = if little {
        usize::from(read_le_u16(input, offset)?)
    } else {
        usize::from(read_be_u16(input, offset)?)
    };
    let table = offset
        .checked_add(2)
        .and_then(|value| value.checked_add(count.checked_mul(12)?))
        .ok_or_else(|| invalid("TIFF IFD size overflows"))?;
    if table + 4 > input.len() {
        return Err(invalid("TIFF IFD entries are truncated"));
    }
    Ok(())
}

fn looks_like_tga(input: &[u8]) -> bool {
    input.len() >= 18 && input[1] <= 1 && [1, 2, 3, 9, 10, 11].contains(&input[2]) && input[16] != 0
}
