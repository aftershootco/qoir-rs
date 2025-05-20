use std::sync::Arc;

use crate::bindings::{qoir_decode_result, qoir_encode_result, qoir_pixel_format, qoir_rectangle};

/// Represents errors that can occur during QOIR encoding or decoding.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// An invalid parameter was provided to a function.
    #[error("Invalid parameter")]
    InvalidParameter,
    /// Decoding of QOIR data failed. Contains a message from the C library.
    #[error("Decoding failed: {0}")]
    DecodingFailed(String),
    /// Encoding to QOIR data failed. Contains a message from the C library.
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),
    /// The specified file could not be found.
    #[error("File not found")]
    FileNotFound,
    /// An I/O error occurred during file reading or writing.
    #[error("I/O error occurred")]
    IoError,
}

/// A rectangle, defined by its top-left (x0, y0) and bottom-right (x1, y1) coordinates.
/// The low bounds are inclusive, high bounds are exclusive.
pub type Rectangle = qoir_rectangle;

// This is the memory allocated for all the fields in this struct
// allocated in one place by the C library to avoid fragmentation.
pub(crate) struct DecodedResult {
    pub(crate) result: qoir_decode_result,
}

unsafe impl Send for DecodedResult {}
unsafe impl Sync for DecodedResult {}

impl DecodedResult {
    pub fn new(result: qoir_decode_result) -> Self {
        DecodedResult { result }
    }
}

impl Drop for DecodedResult {
    fn drop(&mut self) {
        unsafe {
            if !self.result.owned_memory.is_null() {
                libc::free(self.result.owned_memory);
            }
        }
    }
}

pub(crate) struct EncodedResult {
    pub(crate) result: qoir_encode_result,
}

unsafe impl Send for EncodedResult {}
unsafe impl Sync for EncodedResult {}

impl EncodedResult {
    pub fn new(result: qoir_encode_result) -> Self {
        EncodedResult { result }
    }
}

impl Drop for EncodedResult {
    fn drop(&mut self) {
        unsafe {
            if !self.result.owned_memory.is_null() {
                libc::free(self.result.owned_memory);
            }
        }
    }
}

/// Represents the different pixel formats supported by QOIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// Invalid pixel format.
    Invalid = 0x00,
    /// 4 bytes per pixel: B, G, R, then X (ignored).
    BGRX = 0x01,
    /// 4 bytes per pixel: B, G, R, then A (alpha). Non-premultiplied alpha.
    BGRANonPremul = 0x02,
    /// 4 bytes per pixel: B, G, R, then A (alpha). Premultiplied alpha.
    BGRAPremul = 0x03,
    /// 3 bytes per pixel: B, G, R.
    BGR = 0x11,
    /// 4 bytes per pixel: R, G, B, then X (ignored).
    RGBX = 0x21,
    /// 4 bytes per pixel: R, G, B, then A (alpha). Non-premultiplied alpha.
    RGBANonPremul = 0x22,
    /// 4 bytes per pixel: R, G, B, then A (alpha). Premultiplied alpha.
    RGBAPremul = 0x23,
    /// 3 bytes per pixel: R, G, B.
    RGB = 0x31,
    // MaskForAlphaTransperency = 0x03, // Internal C library detail
    // MaskForColorModel = 0x0C,        // Internal C library detail
}

#[allow(non_snake_case, unused_variables)]
impl From<qoir_pixel_format> for PixelFormat {
    fn from(value: qoir_pixel_format) -> Self {
        match value {
            0x00 => PixelFormat::Invalid,
            0x01 => PixelFormat::BGRX,
            0x02 => PixelFormat::BGRANonPremul,
            0x03 => PixelFormat::BGRAPremul,
            0x11 => PixelFormat::BGR,
            0x21 => PixelFormat::RGBX,
            0x22 => PixelFormat::RGBANonPremul,
            0x23 => PixelFormat::RGBAPremul,
            0x31 => PixelFormat::RGB,
            _ => PixelFormat::Invalid,
        }
    }
}

/// Represents an uncompressed image in memory.
///
/// The `pixels` field is a slice referencing the raw pixel data.
/// The lifetime parameter `'data` ensures that this struct does not outlive the
/// data it points to.
#[derive(Debug, Clone)]
pub struct Image<'data> {
    /// Raw pixel data.
    pub pixels: &'data [u8],
    /// Width of the image in pixels.
    pub width: u32,
    /// Height of the image in pixels.
    pub height: u32,
    /// Pixel format of the image data.
    pub pixel_format: PixelFormat,
    /// Stride (or row size) in bytes for the pixel data.
    pub stride_in_bytes: usize,
}

/// Options for controlling the QOIR decoding process.
#[derive(Debug, Clone)]
pub struct DecodeOptions {
    /// If non-zero, this is the pixel format to use when dynamically allocating
    /// the pixel buffer to decode into. Defaults to `PixelFormat::RGBANonPremul`.
    pub pixel_format: PixelFormat,
    /// Optional clipping rectangle in the source coordinate space.
    pub src_clip_rect: Option<Rectangle>,
    /// Optional clipping rectangle in the destination coordinate space.
    pub dst_clip_rect: Option<Rectangle>,
    /// The X offset (in destination coordinate space) to place the top-left
    /// corner of the decoded source image. The Y axis grows down.
    pub offset_x: i32,
    /// The Y offset (in destination coordinate space) to place the top-left
    /// corner of the decoded source image. The Y axis grows down.
    pub offset_y: i32,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        DecodeOptions {
            pixel_format: PixelFormat::RGBANonPremul,
            src_clip_rect: None,
            dst_clip_rect: None,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

/// Represents a decoded QOIR image.
///
/// This struct holds the decoded image data (`image`) and any embedded metadata.
/// The lifetime parameter `'a` is tied to the lifetime of the underlying buffer
/// from which the image was decoded.
#[derive(Clone)]
pub struct DecodedImage<'a> {
    // This is the memory allocated for all the fields in this struct
    // allocated in one place by the C library to avoid fragmentation.
    #[allow(dead_code)]
    pub(crate) result: Arc<DecodedResult>,

    /// The decoded image data (pixels, dimensions, format).
    pub image: Image<'a>,

    /// Optional embedded CICP (Coding-Independent Code Points) profile data.
    pub cic_profile: Option<&'a [u8]>,
    /// Optional embedded ICC (International Color Consortium) profile data.
    pub icc_profile: Option<&'a [u8]>,
    /// Optional embedded EXIF (Exchangeable image file format) data.
    pub exif: Option<&'a [u8]>,
    /// Optional embedded XMP (Extensible Metadata Platform) data.
    pub xmp: Option<&'a [u8]>,
}

/// Options for controlling the QOIR encoding process.
#[derive(Debug, Clone, Default)]
pub struct EncodeOptions {
    /// Optional CICP (Coding-Independent Code Points) profile data to embed.
    pub cicp_profile: Option<Vec<u8>>,
    /// Optional ICC (International Color Consortium) profile data to embed.
    pub icc_profile: Option<Vec<u8>>,
    /// Optional EXIF (Exchangeable image file format) data to embed.
    pub exif: Option<Vec<u8>>,
    /// Optional XMP (Extensible Metadata Platform) data to embed.
    pub xmp: Option<Vec<u8>>,

    /// Lossiness level for encoding. Ranges from 0 (lossless) to 7 (very lossy).
    /// Defaults to 0 (lossless).
    pub lossiness: u8,

    /// Whether to dither the lossy encoding. This option has no effect if `lossiness` is zero.
    /// Defaults to `false`.
    pub dither: bool,
}

/// Represents an encoded QOIR image buffer.
///
/// The `data` field is a slice referencing the raw encoded QOIR byte data.
/// The lifetime parameter `'a` ensures that this struct does not outlive the
/// data it points to (which is managed by the `result` field).
#[derive(Clone)]
pub struct EncodedBuffer<'a> {
    // This is the memory allocated for all the fields in this struct
    // allocated in one place by the C library to avoid fragmentation.
    #[allow(dead_code)]
    pub(crate) result: Arc<EncodedResult>,

    /// The raw QOIR encoded byte data.
    pub data: &'a [u8],
}
