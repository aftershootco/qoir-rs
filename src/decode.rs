use crate::{
    DecodeOptions, DecodedImage, DecodedResult, Error, Image, PixelFormat, Rectangle,
    bindings::{
        qoir_decode, qoir_decode_options, qoir_decode_pixel_configuration, qoir_decode_result,
    },
};
use std::{io::Read, path::Path, sync::Arc};

/// Decodes QOIR image data from a byte slice.
///
/// # Arguments
///
/// * `data`: A slice of bytes containing the QOIR encoded image data.
/// * `options`: `DecodeOptions` to control the decoding process.
///
/// # Returns
///
/// A `Result` containing the `DecodedImage` or an `Error` if decoding fails.
///
/// # Examples
///
/// ```no_run
/// use qoir_rs::{decode_from_memory, DecodeOptions};
///
/// let qoir_data: &[u8] = &[/* ... QOIR data ... */];
/// let options = DecodeOptions::default();
/// match decode_from_memory(qoir_data, options) {
///     Ok(decoded_image) => {
///         println!("Image decoded: {}x{}", decoded_image.image.width, decoded_image.image.height);
///     }
///     Err(e) => {
///         eprintln!("Decoding failed: {:?}", e);
///     }
/// }
/// ```
pub fn decode_from_memory<'a>(
    data: &'_ [u8],
    options: DecodeOptions,
) -> Result<DecodedImage<'a>, Error> {
    let options = qoir_decode_options {
        pixfmt: options.pixel_format as u32,
        offset_x: options.offset_x,
        offset_y: options.offset_y,
        use_src_clip_rectangle: options.src_clip_rect.is_some(),
        use_dst_clip_rectangle: options.dst_clip_rect.is_some(),
        src_clip_rectangle: options.src_clip_rect.unwrap_or(Rectangle::zero()),
        dst_clip_rectangle: options.dst_clip_rect.unwrap_or(Rectangle::zero()),
        ..Default::default()
    };
    let decoded = unsafe {
        qoir_decode(
            data.as_ptr(),
            data.len(),
            &options as *const qoir_decode_options,
        )
    };

    if !decoded.status_message.is_null() {
        let error_message = (unsafe { std::ffi::CStr::from_ptr(decoded.status_message) })
            .to_string_lossy()
            .into_owned();
        return Err(Error::DecodingFailed(error_message));
    }

    Ok(DecodedImage::new(decoded))
}

/// Decodes a QOIR image from a file path.
///
/// # Arguments
///
/// * `path`: A path to the QOIR image file.
/// * `options`: `DecodeOptions` to control the decoding process.
///
/// # Returns
///
/// A `Result` containing the `DecodedImage` or an `Error` if the file cannot be read or decoding fails.
///
/// # Examples
///
/// ```no_run
/// use qoir_rs::{decode, DecodeOptions};
///
/// match decode("input.qoir", DecodeOptions::default()) {
///     Ok(decoded_image) => {
///         println!("Image decoded: {}x{}", decoded_image.image.width, decoded_image.image.height);
///     }
///     Err(e) => {
///         eprintln!("Decoding failed: {:?}", e);
///     }
/// }
/// ```
pub fn decode<'a>(
    path: impl AsRef<Path>,
    options: DecodeOptions,
) -> Result<DecodedImage<'a>, Error> {
    let path = path.as_ref();
    let file = std::fs::File::open(path).map_err(|_| Error::FileNotFound)?;
    let mut reader = std::io::BufReader::new(file);
    let mut data = Vec::new();
    reader.read_to_end(&mut data).map_err(|_| Error::IoError)?;
    decode_from_memory(&data, options)
}

/// Decodes a QOIR image from a reader.
///
/// # Arguments
///
/// * `reader`: An object implementing `std::io::Read` from which QOIR data will be read.
/// * `options`: `DecodeOptions` to control the decoding process.
///
/// # Returns
///
/// A `Result` containing the `DecodedImage` or an `Error` if reading or decoding fails.
///
/// # Examples
///
/// ```no_run
/// use qoir_rs::{decode_from_reader, DecodeOptions};
/// use std::fs::File;
///
/// let file = File::open("input.qoir").expect("Failed to open file");
/// let options = DecodeOptions::default();
/// match decode_from_reader(file, options) {
///     Ok(decoded_image) => {
///         println!("Image decoded: {}x{}", decoded_image.image.width, decoded_image.image.height);
///     }
///     Err(e) => {
///         eprintln!("Decoding failed: {:?}", e);
///     }
/// }
/// ```
pub fn decode_from_reader<'a>(
    reader: impl Read,
    options: DecodeOptions,
) -> Result<DecodedImage<'a>, Error> {
    let mut data = Vec::new();
    let mut reader = std::io::BufReader::new(reader);
    reader.read_to_end(&mut data).map_err(|_| Error::IoError)?;
    decode_from_memory(&data, options)
}

/// Decodes basic metadata (width, height, pixel format) from QOIR image data.
///
/// This function is faster than full decoding if only metadata is needed.
///
/// # Arguments
///
/// * `data`: A slice of bytes containing the QOIR encoded image data.
///
/// # Returns
///
/// A `Result` containing a tuple `(width, height, PixelFormat)` or an `Error` if metadata decoding fails.
///
/// # Examples
///
/// ```no_run
/// use qoir_rs::decode_basic_metadata;
///
/// let qoir_data: &[u8] = &[/* ... QOIR data ... */];
/// match decode_basic_metadata(qoir_data) {
///     Ok((width, height, pixel_format)) => {
///         println!("Image metadata: {}x{}, Format: {:?}", width, height, pixel_format);
///     }
///     Err(e) => {
///         eprintln!("Metadata decoding failed: {:?}", e);
///     }
/// }
/// ```
pub fn decode_basic_metadata(data: &[u8]) -> Result<(u32, u32, PixelFormat), Error> {
    let decoded = unsafe { qoir_decode_pixel_configuration(data.as_ptr(), data.len()) };

    if !decoded.status_message.is_null() {
        let error_message = (unsafe { std::ffi::CStr::from_ptr(decoded.status_message) })
            .to_string_lossy()
            .into_owned();
        return Err(Error::DecodingFailed(error_message));
    }

    let pixel_format = PixelFormat::from(decoded.dst_pixcfg.pixfmt);
    let width = decoded.dst_pixcfg.width_in_pixels;
    let height = decoded.dst_pixcfg.height_in_pixels;

    Ok((width, height, pixel_format))
}

impl DecodedImage<'_> {
    /// Creates a new `DecodedImage` from the raw `qoir_decode_result`.
    ///
    /// This is an internal function.
    pub(crate) fn new(data: qoir_decode_result) -> Self {
        let result = Arc::new(DecodedResult::new(data));

        let pixels = unsafe {
            // NOTE: Verify this
            std::slice::from_raw_parts(
                result.result.dst_pixbuf.data as *const u8,
                result.result.dst_pixbuf.pixcfg.width_in_pixels as usize
                    * result.result.dst_pixbuf.pixcfg.height_in_pixels as usize
                    * result.result.dst_pixbuf.stride_in_bytes,
            )
        };

        let pixel_format = PixelFormat::from(result.result.dst_pixbuf.pixcfg.pixfmt);
        let width = result.result.dst_pixbuf.pixcfg.width_in_pixels;
        let height = result.result.dst_pixbuf.pixcfg.height_in_pixels;
        let stride_in_bytes = result.result.dst_pixbuf.stride_in_bytes;

        let cic_profile = if !result.result.metadata_cicp_ptr.is_null() {
            Some(unsafe {
                std::slice::from_raw_parts(
                    result.result.metadata_cicp_ptr,
                    result.result.metadata_cicp_len,
                )
            })
        } else {
            None
        };

        let icc_profile = if !result.result.metadata_iccp_ptr.is_null() {
            Some(unsafe {
                std::slice::from_raw_parts(
                    result.result.metadata_iccp_ptr,
                    result.result.metadata_iccp_len,
                )
            })
        } else {
            None
        };

        let exif = if !result.result.metadata_exif_ptr.is_null() {
            Some(unsafe {
                std::slice::from_raw_parts(
                    result.result.metadata_exif_ptr,
                    result.result.metadata_exif_len,
                )
            })
        } else {
            None
        };

        let xmp = if !result.result.metadata_xmp_ptr.is_null() {
            Some(unsafe {
                std::slice::from_raw_parts(
                    result.result.metadata_xmp_ptr,
                    result.result.metadata_xmp_len,
                )
            })
        } else {
            None
        };

        let image = Image {
            pixels,
            width,
            height,
            pixel_format,
            stride_in_bytes,
        };

        Self {
            result,
            image,
            cic_profile,
            icc_profile,
            exif,
            xmp,
        }
    }
}
