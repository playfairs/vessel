# Executable image polyglot analysis

This project does not implement a genuine native macOS executable-image polyglot for PNG.

## Proven invariant

The macOS loader requires a Mach-O header at file offset zero for a native executable. The canonical Mach-O magic values are:

- `0xfeedfacf` (32-bit)
- `0xfeedface` (32-bit little-endian)
- `0xcefaedfe` (64-bit)
- `0xcafebabe` (64-bit)

A PNG begins with the sequence:

```text
89 50 4e 47 0d 0a 1a 0a
```

Those signatures are mutually exclusive at offset zero. The implementation therefore rejects native executable polyglot claims rather than manufacturing an impossible file layout.

## Real experiments

The following concrete commands were run against the repository artifacts:

```sh
file examples/images/png/image.png
file examples/binaries/hello

cat examples/images/png/image.png examples/binaries/hello > /tmp/png_plus_macho
cat examples/binaries/hello examples/images/png/image.png > /tmp/macho_plus_png

file /tmp/png_plus_macho
file /tmp/macho_plus_png

chmod +x /tmp/png_plus_macho
/tmp/png_plus_macho

chmod +x /tmp/macho_plus_png
/tmp/macho_plus_png
```

Observed results:

- `examples/images/png/image.png` is recognized as a PNG.
- `examples/binaries/hello` is recognized as a Mach-O 64-bit arm64 executable.
- `PNG + Mach-O` is recognized by macOS as a Mach-O executable, because the loader inspects byte 0 and finds the Mach-O magic.
- `Mach-O + PNG` is recognized as a PNG image, because the PNG signature at byte 0 wins for image parsers.
- the `Mach-O + PNG` case fails with `exec format error` when executed as a program.

The important point is that the loader is not a dual parser. It selects one interpretation based on the start of the file.

## Why this matters

A valid executable-image polyglot must satisfy both of these conditions simultaneously:

1. a macOS loader must accept the file as an executable, and
2. an image parser must accept the file as a valid PNG.

Because the PNG signature and Mach-O magic are incompatible at offset zero, a file cannot satisfy both under the standard macOS loader contract without changing the file format semantics or using a loader-specific exception that is not a standard PNG image file.

## Closest valid architecture

The supported architecture is the secure extracted-payload path:

```text
image file
  -> PNG parser validates image
  -> veSL parser validates payload metadata
  -> payload is written to a temporary executable
  -> execve-like direct process launch with forwarded arguments
```

This is the design implemented by `vessel run` and is the strongest architecture that is both valid and secure under the current macOS constraints.
