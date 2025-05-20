use std::{io::Write, path::Path, sync::Arc};

use crate::{
    EncodeOptions, EncodedBuffer, EncodedResult, Error, Image,
    bindings::{
        qoir_encode, qoir_encode_options, qoir_encode_result, qoir_pixel_buffer,
        qoir_pixel_buffer_struct, qoir_pixel_configuration,
    },
};

/// Encodes an `Image` into QOIR format in memory.
///
/// # Arguments
///
/// * `image`: The `Image` to encode.
/// * `options`: `EncodeOptions` to control the encoding process.
///
/// # Returns
///
/// A `Result` containing the `EncodedBuffer` or an `Error` if encoding fails.
///
/// # Examples
///
/// ```no_run
/// use qoir_rs::{encode_to_memory, EncodeOptions, Image, PixelFormat};
///
/// // Assuming `pixels`, `width`, and `height` are defined
/// let image_data = Image {
///     pixels: &pixels,
///     width,
///     height,
///     pixel_format: PixelFormat::RGBANonPremul,
///     stride_in_bytes: (width * 4) as usize, // For RGBA
/// };
/// let options = EncodeOptions::default();
/// match encode_to_memory(image_data, options) {
///     Ok(encoded_buffer) => {
///         println!("Image encoded to {} bytes", encoded_buffer.data.len());
///     }
///     Err(e) => {
///         eprintln!("Encoding failed: {:?}", e);
///     }
/// }
/// ```
pub fn encode_to_memory<'a>(
    image: Image<'_>,
    options: EncodeOptions,
) -> Result<EncodedBuffer<'a>, Error> {
    let options = qoir_encode_options {
        metadata_cicp_ptr: options
            .cicp_profile
            .as_deref()
            .map_or(std::ptr::null(), |s| s.as_ptr()),
        metadata_cicp_len: options.cicp_profile.as_deref().map_or(0, |s| s.len()),
        metadata_iccp_ptr: options
            .icc_profile
            .as_deref()
            .map_or(std::ptr::null(), |s| s.as_ptr()),
        metadata_iccp_len: options.icc_profile.as_deref().map_or(0, |s| s.len()),
        metadata_exif_ptr: options
            .exif
            .as_deref()
            .map_or(std::ptr::null(), |s| s.as_ptr()),
        metadata_exif_len: options.exif.as_deref().map_or(0, |s| s.len()),
        metadata_xmp_ptr: options
            .xmp
            .as_deref()
            .map_or(std::ptr::null(), |s| s.as_ptr()),
        metadata_xmp_len: options.xmp.as_deref().map_or(0, |s| s.len()),
        lossiness: options.lossiness as u32,
        dither: options.dither,
        ..Default::default()
    };

    let pix_buff = qoir_pixel_buffer {
        stride_in_bytes: image.stride_in_bytes,
        data: image.pixels.as_ptr() as *mut u8,
        pixcfg: qoir_pixel_configuration {
            width_in_pixels: image.width,
            height_in_pixels: image.height,
            pixfmt: image.pixel_format as u32,
        },
    };

    let result = unsafe {
        qoir_encode(
            &pix_buff as *const qoir_pixel_buffer_struct,
            &options as *const qoir_encode_options,
        )
    };

    if !result.status_message.is_null() {
        let error_message = (unsafe { std::ffi::CStr::from_ptr(result.status_message) })
            .to_string_lossy()
            .into_owned();
        return Err(Error::EncodingFailed(error_message));
    }

    Ok(EncodedBuffer::new(result))
}

/// Encodes an `Image` into QOIR format and writes it to a `Write` implementor.
///
/// # Arguments
///
/// * `image`: The `Image` to encode.
/// * `options`: `EncodeOptions` to control the encoding process.
/// * `writer`: An object implementing `std::io::Write` to which QOIR data will be written.
///
/// # Returns
///
/// A `Result` containing the `EncodedBuffer` (which also holds a reference to the encoded data)
/// or an `Error` if encoding or writing fails.
///
/// # Examples
///
/// ```no_run
/// use qoir_rs::{encode_to_writer, EncodeOptions, Image, PixelFormat};
/// use std::fs::File;
///
/// // Assuming `pixels`, `width`, and `height` are defined
/// let image_data = Image {
///     pixels: &pixels,
///     width,
///     height,
///     pixel_format: PixelFormat::RGBANonPremul,
///     stride_in_bytes: (width * 4) as usize, // For RGBA
/// };
/// let options = EncodeOptions::default();
/// let mut file = File::create("output.qoir").expect("Failed to create file");
/// match encode_to_writer(image_data, options, &mut file) {
///     Ok(_) => {
///         println!("Image encoded and written to output.qoir");
///     }
///     Err(e) => {
///         eprintln!("Encoding or writing failed: {:?}", e);
///     }
/// }
/// ```
pub fn encode_to_writer<'a>(
    image: Image<'_>,
    options: EncodeOptions,
    writer: impl Write,
) -> Result<EncodedBuffer<'a>, Error> {
    let encoded_buffer = encode_to_memory(image, options)?;
    let mut writer = std::io::BufWriter::new(writer);
    writer
        .write_all(encoded_buffer.data)
        .map_err(|_| Error::IoError)?;
    Ok(encoded_buffer)
}

/// Encodes an `Image` into QOIR format and writes it to a file path.
///
/// # Arguments
///
/// * `image`: The `Image` to encode.
/// * `options`: `EncodeOptions` to control the encoding process.
/// * `path`: A path to the file where the QOIR image will be saved.
///
/// # Returns
///
/// A `Result` containing the `EncodedBuffer` (which also holds a reference to the encoded data)
/// or an `Error` if encoding or writing fails.
///
/// # Examples
///
/// ```no_run
/// use qoir_rs::{encode_to_file, EncodeOptions, Image, PixelFormat};
///
/// // Assuming `pixels`, `width`, and `height` are defined
/// let image_data = Image {
///     pixels: &pixels,
///     width,
///     height,
///     pixel_format: PixelFormat::RGBANonPremul,
///     stride_in_bytes: (width * 4) as usize, // For RGBA
/// };
/// let options = EncodeOptions::default();
/// match encode_to_file(image_data, options, "output.qoir") {
///     Ok(_) => {
///         println!("Image encoded and saved to output.qoir");
///     }
///     Err(e) => {
///         eprintln!("Encoding or saving failed: {:?}", e);
///     }
/// }
/// ```
pub fn encode_to_file<'a>(
    image: Image<'_>,
    options: EncodeOptions,
    path: impl AsRef<Path>,
) -> Result<EncodedBuffer<'a>, Error> {
    let file = std::fs::File::create(path).map_err(|_| Error::IoError)?;
    encode_to_writer(image, options, file)
}

impl EncodedBuffer<'_> {
    /// Creates a new `EncodedBuffer` from the raw `qoir_encode_result`.
    ///
    /// This is an internal function.
    pub(crate) fn new(buffer: qoir_encode_result) -> Self {
        let buffer = EncodedResult::new(buffer);
        let data = unsafe {
            std::slice::from_raw_parts(buffer.result.dst_ptr as *const u8, buffer.result.dst_len)
        };

        EncodedBuffer {
            result: Arc::new(buffer),
            data,
        }
    }
}
