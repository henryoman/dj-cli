# Release Build Guide

This document summarizes the release preparation for `dj-cli`.

## Build Targets

The project builds on these platforms:

- **Linux (x86_64-unknown-linux-gnu)**
- **Windows (x86_64-pc-windows-gnu)**
- **macOS (x86_64-apple-darwin)** — build requires macOS toolchain and cannot be produced in the current Linux container.

## Prerequisites

- Rust 1.78 or later
- `yt-dlp` and `ffmpeg` available at runtime

For cross-compiling to Windows on Linux, install the target and toolchain:

```bash
rustup target add x86_64-pc-windows-gnu
apt-get update && apt-get install -y mingw-w64
```

## Build Commands

### Linux
```bash
cargo build --release
```
Output: `target/release/dj-cli`

### Windows (cross-compiled)
```bash
cargo build --release --target x86_64-pc-windows-gnu
```
Output: `target/x86_64-pc-windows-gnu/release/dj-cli.exe`

### macOS
```bash
cargo build --release --target x86_64-apple-darwin
```
> Requires the macOS SDK and linker; cross-compilation failed in this environment because the `cc` linker does not understand `-arch` flags.

## Packaging

Create archives for GitHub release:

```bash
# Linux tarball
cd target/release
 tar czf dj-cli-x86_64-unknown-linux-gnu.tar.gz dj-cli

# Windows zip
cd target/x86_64-pc-windows-gnu/release
 zip dj-cli-x86_64-pc-windows-gnu.zip dj-cli.exe
```

Upload these archives as release assets on GitHub. Include a note that macOS users should build from source or use the Rust crate.

## Tests

Run the test suite before releasing:
```bash
cargo test
```

The current test run completed successfully with zero tests.

