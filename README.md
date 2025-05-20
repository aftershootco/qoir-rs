# QOIR-RS

Rust bindings for the [QOIR (Quite OK Image Format)](https://github.com/nigeltao/qoir) library.

This crate provides a safe interface to the C `qoir` library, allowing for encoding and decoding of QOIR images. It also includes a basic command-line interface (CLI) tool for common QOIR operations.

## Features

- Decode QOIR images from memory, files, or readers.
- Encode images to QOIR format into memory, files, or writers.
- Access to image metadata (width, height, pixel format).
- Support for various pixel formats.
- Control over decoding options like clipping and offset.
- Control over encoding options like lossiness and dithering.
- A simple CLI for encoding, decoding, and inspecting QOIR files.

## Getting Started

Add `qoir-rs` to your `Cargo.toml`:

```toml
[dependencies]
qoir-rs = "0.1.0" # Replace with the latest version
```

## Library Usage Examples

### Decoding an image from a file

```rust
use qoir_rs::{decode, DecodeOptions, Error};
use std::path::Path;

fn main() -> Result<(), Error> {
    let options = DecodeOptions::default();
    let decoded_image = decode(Path::new("input.qoir"), options)?;

    println!("Image decoded: {}x{}", decoded_image.image.width, decoded_image.image.height);
    println!("Pixel format: {:?}", decoded_image.image.pixel_format);

    // Access pixel data
    let pixels = decoded_image.image.pixels;
    // ... process pixels ...

    Ok(())
}
```

### Encoding an image to a file

```rust
use qoir_rs::{encode, EncodeOptions, Image, PixelFormat, Error};
use std::path::Path;

fn main() -> Result<(), Error> {
    // Example: Create a dummy 10x10 RGB image (all black)
    let width = 10;
    let height = 10;
    let pixel_data = vec![0u8; (width * height * 3) as usize]; // 3 bytes per pixel for RGB

    let image = Image {
        pixels: &pixel_data,
        width,
        height,
        pixel_format: PixelFormat::RGB,
        stride_in_bytes: (width * 3) as usize,
    };

    let options = EncodeOptions {
        lossiness: 0, // Lossless
        ..Default::default()
    };

    encode(image, options, Path::new("output.qoir"))?;
    println!("Image encoded and saved to output.qoir");

    Ok(())
}
```

### Decoding basic image metadata

If you only need the image dimensions and pixel format without decoding the full pixel data:

```rust
use qoir_rs::{decode_basic_metadata, Error};
use std::fs;

fn main() -> Result<(), Error> {
    let qoir_data = fs::read("input.qoir").expect("Failed to read QOIR file");
    let (width, height, pixel_format) = decode_basic_metadata(&qoir_data)?;

    println!("Image metadata: {}x{}, Format: {:?}", width, height, pixel_format);

    Ok(())
}
```

For more detailed examples, see the documentation for the specific functions and structs within the `src/lib.rs` file and the `tests` directory.

## Command-Line Interface (CLI)

This crate also builds a CLI tool named `qoir-rs`.

### Building the CLI

```bash
cargo build --release
```
The executable will be in `target/release/qoir-rs`.

### CLI Usage

