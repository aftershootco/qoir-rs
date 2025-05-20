use std::sync::Arc;

use crate::bindings::{qoir_decode_result, qoir_encode_result, qoir_pixel_format, qoir_rectangle};

#[derive(Debug, Clone)]
pub enum Error {
    InvalidParameter,
    DecodingFailed(String),
    EncodingFailed(String),
    FileNotFound,
    IoError,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    Invalid = 0x00,
    BGRX = 0x01,
    BGRANonPremul = 0x02,
    BGRAPremul = 0x03,
    BGR = 0x11,
    RGBX = 0x21,
    RGBANonPremul = 0x22,
    RGBAPremul = 0x23,
    RGB = 0x31,
    // MaskForAlphaTransperency = 0x03,
    // MaskForColorModel = 0x0C,
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

#[derive(Debug, Clone)]
pub struct Image<'data> {
    // This is the decoded pixels data.
    // This could also be translated as a enum
    // with a variant for each pixel format.
    // But this way its more convinient to use
    // in a lot of cases.
    pub pixels: &'data [u8],

    // The width and height of the decoded image.
    pub width: u32,
    // The width and height of the decoded image.
    pub height: u32,

    // The pixel format of the decoded image.
    pub pixel_format: PixelFormat,

    pub stride_in_bytes: usize,
}

pub struct DecodeOptions {
    // If non-zero, this is the pixel format to use when dynamically allocating
    // the pixel buffer to decode into.
    pub pixel_format: PixelFormat,
    // Clipping rectangles, in the destination or source (or both) coordinate
    // spaces.
    pub src_clip_rect: Option<Rectangle>,
    pub dst_clip_rect: Option<Rectangle>,
    // The position (in destination coordinate space) to place the top-left
    // corner of the decoded source image. The Y axis grows down.
    pub offset_x: i32,
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

#[derive(Clone)]
pub struct DecodedImage<'a> {
    // This is the memory allocated for all the fields in this struct
    // allocated in one place by the C library to avoid fragmentation.
    #[allow(dead_code)]
    pub(crate) result: Arc<DecodedResult>,

    // The image data
    pub image: Image<'a>,

    // The embedded metadata (optional)
    pub cic_profile: Option<&'a [u8]>,
    pub icc_profile: Option<&'a [u8]>,
    pub exif: Option<&'a [u8]>,
    pub xmp: Option<&'a [u8]>,
}

#[derive(Debug, Clone, Default)]
pub struct EncodeOptions {
    // Optional metadata chunks.
    pub cicp_profile: Option<Vec<u8>>,
    pub icc_profile: Option<Vec<u8>>,
    pub exif: Option<Vec<u8>>,
    pub xmp: Option<Vec<u8>>,

    // Lossiness ranges from 0 (lossless) to 7 (very lossy), inclusive.
    pub lossiness: u8,

    // Whether to dither the lossy encoding. This option has no effect if
    // lossiness is zero.
    //
    // The dithering algorithm is relatively simple. Fancier algorithms like
    // https://nigeltao.github.io/blog/2022/gamma-aware-ordered-dithering.html
    // can produce higher quality results, especially for lossiness levels at 6
    // or 7 re overall brightness, but they are out of scope of this library.
    pub dither: bool,
}

#[derive(Clone)]
pub struct EncodedBuffer<'a> {
    // This is the memory allocated for all the fields in this struct
    // allocated in one place by the C library to avoid fragmentation.
    #[allow(dead_code)]
    pub(crate) result: Arc<EncodedResult>,

    // The encoded image data
    pub data: &'a [u8],
}
