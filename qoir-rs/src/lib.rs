//! # qoir-rs
//!
//! Rust bindings for the QOIR (Quite OK Image Format) library.
//!
//! This crate provides a safe interface to the C `qoir` library, allowing for encoding and decoding of QOIR images.
//!
//! ## Features
//!
//! - Decode QOIR images from memory, files, or readers.
//! - Encode images to QOIR format into memory, files, or writers.
//! - Access to image metadata (width, height, pixel format).
//! - Support for various pixel formats.
//! - Control over decoding options like clipping and offset.
//! - Control over encoding options like lossiness and dithering.
//!
//! ## Getting Started
//!
//! Add `qoir-rs` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! qoir-rs = "0.1.0" # Replace with the latest version
//! ```
//!
//! ## Examples
//!
//! ### Decoding an image from a file
//!
//! ```no_run
//! use qoir_rs::{decode, DecodeOptions, Error};
//!
//! fn main() -> Result<(), Error> {
//!     let options = DecodeOptions::default();
//!     let decoded_image = decode("input.qoir", options)?;
//!
//!     println!("Image decoded: {}x{}", decoded_image.image.width, decoded_image.image.height);
//!     println!("Pixel format: {:?}", decoded_image.image.pixel_format);
//!
//!     // Access pixel data
//!     let pixels = decoded_image.image.pixels;
//!     // ... process pixels ...
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Encoding an image to a file
//!
//! ```no_run
//! use qoir_rs::{encode, EncodeOptions, Image, PixelFormat, Error};
//!
//! fn main() -> Result<(), Error> {
//!     // Example: Create a dummy 10x10 RGB image (all black)
//!     let width = 10;
//!     let height = 10;
//!     let pixel_data = vec![0u8; (width * height * 3) as usize]; // 3 bytes per pixel for RGB
//!
//!     let image = Image {
//!         pixels: &pixel_data,
//!         width,
//!         height,
//!         pixel_format: PixelFormat::RGB,
//!         stride_in_bytes: (width * 3) as usize,
//!     };
//!
//!     let options = EncodeOptions {
//!         lossiness: 0, // Lossless
//!         ..Default::default()
//!     };
//!
//!     encode(image, options, "output.qoir")?;
//!     println!("Image encoded and saved to output.qoir");
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Decoding basic image metadata
//!
//! If you only need the image dimensions and pixel format without decoding the full pixel data:
//!
//! ```no_run
//! use qoir_rs::{decode_basic_metadata, Error};
//!
//! fn main() -> Result<(), Error> {
//!     let qoir_data: &[u8] = &[/* ... QOIR data ... */]; // Load your QOIR data here
//!     let (width, height, pixel_format) = decode_basic_metadata(qoir_data)?;
//!
//!     println!("Image metadata: {}x{}, Format: {:?}", width, height, pixel_format);
//!
//!     Ok(())
//! }
//! ```
//!
//! For more detailed examples, see the documentation for the specific functions and structs.

mod bindings;

mod types;
pub use types::*;

mod decode;
pub use decode::*;

mod encode;
pub use encode::*;
