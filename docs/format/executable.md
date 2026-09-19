# Executable-image boundary

The canonical payload `examples/binaries/hello` is a native arm64 Mach-O executable. `vessel run` extracts it from a verified Vessel PNG into an unpredictable temporary directory, sets mode `0700`, invokes it directly with `std::process::Command`, forwards argument boundaries and the current environment, waits for completion, and returns its exit status.

Vessel also supports macOS application bundle payloads. `vessel create` serializes a `.app` directory as a versioned bundle payload, preserving its relative files, directories, Unix modes, and relative symlinks. `vessel run` reconstructs that layout and launches `Contents/MacOS/<bundle-name>` from it. This keeps dyld paths such as `@executable_path/../Frameworks/UnityPlayer.dylib` valid because the executable and its Frameworks directory retain their original relative locations.

`vessel create` validates the source image and payload, creates the canonical Vessel container, validates the resulting PNG again, and writes it. The result is a valid PNG container and is not itself executable.

A normal macOS executable must begin with a Mach-O magic value. PNG, JPEG, WebP, GIF, BMP, Netpbm, TGA, and TIFF require incompatible format signatures at byte zero. A file made by concatenating an image, a Mach-O, or a payload is not a valid solution: it does not satisfy both loaders and does not provide a structured locator. Vessel therefore reports native executable-polyglot support as unavailable instead of claiming unsupported behavior based on permissive viewers or trailing bytes.