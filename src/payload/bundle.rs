use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use crate::error::VesselError;

const MAGIC: &[u8; 8] = b"VSLBND01";
const VERSION: u16 = 1;
const FILE: u8 = 1;
const DIRECTORY: u8 = 2;
const SYMLINK: u8 = 3;

struct Entry {
    kind: u8,
    mode: u32,
    path: Vec<u8>,
    data: Vec<u8>,
}

pub fn pack(path: &Path) -> Result<Vec<u8>, VesselError> {
    pack_with_progress(path, &mut |_, _| {})
}

pub fn pack_with_progress(
    path: &Path,
    progress: &mut dyn FnMut(usize, &Path),
) -> Result<Vec<u8>, VesselError> {
    if !path.is_dir() {
        return Err(malformed("bundle source is not a directory"));
    }
    let name = path
        .file_stem()
        .ok_or_else(|| malformed("bundle has no name"))?;
    let entrypoint = PathBuf::from("Contents/MacOS").join(name);
    if !path.join(&entrypoint).is_file() {
        return Err(malformed("bundle entrypoint is missing"));
    }
    let mut entries = Vec::new();
    let mut count = 0;
    collect(path, Path::new(""), &mut entries, progress, &mut count)?;
    let entrypoint = entrypoint
        .to_str()
        .ok_or_else(|| malformed("bundle entrypoint is not UTF-8"))?;
    let entrypoint_len =
        u16::try_from(entrypoint.len()).map_err(|_| malformed("bundle entrypoint is too long"))?;
    let count =
        u32::try_from(entries.len()).map_err(|_| malformed("bundle has too many entries"))?;
    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&VERSION.to_be_bytes());
    output.extend_from_slice(&count.to_be_bytes());
    output.extend_from_slice(&entrypoint_len.to_be_bytes());
    output.extend_from_slice(entrypoint.as_bytes());
    for entry in entries {
        let path_len =
            u32::try_from(entry.path.len()).map_err(|_| malformed("bundle path is too long"))?;
        let data_len =
            u64::try_from(entry.data.len()).map_err(|_| malformed("bundle entry is too large"))?;
        output.push(entry.kind);
        output.extend_from_slice(&entry.mode.to_be_bytes());
        output.extend_from_slice(&path_len.to_be_bytes());
        output.extend_from_slice(&data_len.to_be_bytes());
        output.extend_from_slice(&entry.path);
        output.extend_from_slice(&entry.data);
    }
    Ok(output)
}

pub fn pack_if_app_executable(path: &Path) -> Result<Option<Vec<u8>>, VesselError> {
    pack_if_app_executable_with_progress(path, &mut |_, _| {})
}

pub fn pack_if_app_executable_with_progress(
    path: &Path,
    progress: &mut dyn FnMut(usize, &Path),
) -> Result<Option<Vec<u8>>, VesselError> {
    if path.is_dir() {
        return Ok(Some(pack_with_progress(path, progress)?));
    }
    if !path.is_file() {
        return Ok(None);
    }
    let macos = path
        .parent()
        .filter(|parent| parent.file_name().is_some_and(|name| name == "MacOS"));
    let contents = macos
        .and_then(Path::parent)
        .filter(|parent| parent.file_name().is_some_and(|name| name == "Contents"));
    let app = contents.and_then(Path::parent).filter(|parent| {
        parent
            .extension()
            .is_some_and(|extension| extension == "app")
    });
    let Some(app) = app else {
        return Ok(None);
    };
    let expected_name = app
        .file_stem()
        .ok_or_else(|| malformed("bundle has no name"))?;
    if path.file_name() != Some(expected_name) {
        return Err(malformed(
            "bundle executable name does not match the bundle name",
        ));
    }
    Ok(Some(pack_with_progress(app, progress)?))
}

pub fn is_bundle(payload: &[u8]) -> bool {
    payload.starts_with(MAGIC)
}

pub fn unpack(payload: &[u8], destination: &Path) -> Result<PathBuf, VesselError> {
    let mut cursor = 0;
    if take(payload, &mut cursor, MAGIC.len())? != MAGIC {
        return Err(malformed("bundle magic is invalid"));
    }
    let version = read_u16(payload, &mut cursor)?;
    if version != VERSION {
        return Err(VesselError::UnsupportedVersion(version));
    }
    let count = usize::try_from(read_u32(payload, &mut cursor)?)
        .map_err(|_| malformed("bundle entry count is invalid"))?;
    let entrypoint_len = usize::from(read_u16(payload, &mut cursor)?);
    let entrypoint = String::from_utf8(take(payload, &mut cursor, entrypoint_len)?.to_vec())
        .map_err(|_| malformed("bundle entrypoint is not UTF-8"))?;
    validate_relative(Path::new(&entrypoint))?;
    for _ in 0..count {
        let kind = take(payload, &mut cursor, 1)?[0];
        if ![FILE, DIRECTORY, SYMLINK].contains(&kind) {
            return Err(malformed("bundle entry type is invalid"));
        }
        let mode = read_u32(payload, &mut cursor)?;
        let path_len = usize::try_from(read_u32(payload, &mut cursor)?)
            .map_err(|_| malformed("bundle path length is invalid"))?;
        let data_len = usize::try_from(read_u64(payload, &mut cursor)?)
            .map_err(|_| malformed("bundle data length is invalid"))?;
        let path = String::from_utf8(take(payload, &mut cursor, path_len)?.to_vec())
            .map_err(|_| malformed("bundle path is not UTF-8"))?;
        let relative = Path::new(&path);
        validate_relative(relative)?;
        let data = take(payload, &mut cursor, data_len)?;
        let target = destination.join(relative);
        match kind {
            DIRECTORY => {
                fs::create_dir_all(&target)
                    .map_err(|source| crate::error::io_error(&target, source))?;
                set_mode(&target, mode)?;
            }
            FILE => {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|source| crate::error::io_error(parent, source))?;
                }
                fs::write(&target, data)
                    .map_err(|source| crate::error::io_error(&target, source))?;
                set_mode(&target, mode)?;
            }
            SYMLINK => {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|source| crate::error::io_error(parent, source))?;
                }
                let link = String::from_utf8(data.to_vec())
                    .map_err(|_| malformed("bundle symlink target is not UTF-8"))?;
                validate_relative(Path::new(&link))?;
                #[cfg(unix)]
                std::os::unix::fs::symlink(link, &target)
                    .map_err(|source| crate::error::io_error(&target, source))?;
                #[cfg(not(unix))]
                return Err(malformed(
                    "bundle symlinks are unsupported on this platform",
                ));
            }
            _ => unreachable!(),
        }
    }
    if cursor != payload.len() {
        return Err(malformed("bundle has trailing data"));
    }
    Ok(destination.join(entrypoint))
}

fn collect(
    root: &Path,
    relative: &Path,
    entries: &mut Vec<Entry>,
    progress: &mut dyn FnMut(usize, &Path),
    count: &mut usize,
) -> Result<(), VesselError> {
    let directory = root.join(relative);
    let mut children = fs::read_dir(&directory)
        .map_err(|source| crate::error::io_error(&directory, source))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| crate::error::io_error(&directory, source))?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let child_relative = relative.join(child.file_name());
        let metadata = fs::symlink_metadata(child.path())
            .map_err(|source| crate::error::io_error(&child.path(), source))?;
        let mode = file_mode(&metadata);
        let path = child_relative
            .to_str()
            .ok_or_else(|| malformed("bundle path is not UTF-8"))?
            .as_bytes()
            .to_vec();
        if metadata.is_dir() {
            collect(root, &child_relative, entries, progress, count)?;
            entries.push(Entry {
                kind: DIRECTORY,
                mode,
                path,
                data: Vec::new(),
            });
        } else if metadata.file_type().is_symlink() {
            let target = fs::read_link(child.path())
                .map_err(|source| crate::error::io_error(&child.path(), source))?;
            let target = target
                .to_str()
                .ok_or_else(|| malformed("bundle symlink target is not UTF-8"))?;
            validate_relative(Path::new(target))?;
            entries.push(Entry {
                kind: SYMLINK,
                mode,
                path,
                data: target.as_bytes().to_vec(),
            });
        } else if metadata.is_file() {
            let data = fs::read(child.path())
                .map_err(|source| crate::error::io_error(&child.path(), source))?;
            entries.push(Entry {
                kind: FILE,
                mode,
                path,
                data,
            });
        } else {
            return Err(malformed("bundle contains unsupported filesystem entry"));
        }
        *count += 1;
        progress(*count, &child_relative);
    }
    Ok(())
}

fn validate_relative(path: &Path) -> Result<(), VesselError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(malformed("bundle path must be relative and traversal-free"));
    }
    Ok(())
}

fn take<'a>(input: &'a [u8], cursor: &mut usize, length: usize) -> Result<&'a [u8], VesselError> {
    let end = cursor
        .checked_add(length)
        .ok_or_else(|| malformed("bundle offset overflow"))?;
    let bytes = input
        .get(*cursor..end)
        .ok_or_else(|| malformed("bundle is truncated"))?;
    *cursor = end;
    Ok(bytes)
}

fn read_u16(input: &[u8], cursor: &mut usize) -> Result<u16, VesselError> {
    Ok(u16::from_be_bytes(
        take(input, cursor, 2)?.try_into().unwrap(),
    ))
}

fn read_u32(input: &[u8], cursor: &mut usize) -> Result<u32, VesselError> {
    Ok(u32::from_be_bytes(
        take(input, cursor, 4)?.try_into().unwrap(),
    ))
}

fn read_u64(input: &[u8], cursor: &mut usize) -> Result<u64, VesselError> {
    Ok(u64::from_be_bytes(
        take(input, cursor, 8)?.try_into().unwrap(),
    ))
}

fn malformed(message: &str) -> VesselError {
    VesselError::MalformedVessel(message.into())
}

#[cfg(unix)]
fn file_mode(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn file_mode(_metadata: &fs::Metadata) -> u32 {
    0o700
}

fn set_mode(path: &Path, mode: u32) -> Result<(), VesselError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .map_err(|source| crate::error::io_error(path, source))?;
    }
    Ok(())
}
