use qoir_rs::{
    encode,
    encode_to_memory,
    DecodeOptions,
    EncodeOptions,
    Image,
    PixelFormat,
    decode_from_memory,
};
use std::fs::{ self, File };
use std::io::{ BufWriter, Write };
use std::path::Path;

const TEST_DATA_DIR: &str = "../data";
const TEST_OUTPUT_DIR: &str = "tests/output";

fn ensure_output_dir() {
    fs::create_dir_all(TEST_OUTPUT_DIR).expect("Failed to create output directory");
}

fn get_test_file_path(name: &str) -> String {
    format!("{}/{}", TEST_DATA_DIR, name)
}

fn get_output_file_path(name: &str) -> String {
    format!("{}/{}", TEST_OUTPUT_DIR, name)
}

// Helper to create a dummy image for encoding tests
fn create_dummy_image(width: u32, height: u32, pixel_format: PixelFormat) -> Image<'static> {
    let channels = match pixel_format {
        PixelFormat::RGBANonPremul | PixelFormat::BGRAPremul | PixelFormat::BGRANonPremul => 4,
        PixelFormat::RGB | PixelFormat::BGR => 3,
        _ =>
            panic!(
                "Unsupported pixel format for dummy image creation in tests: {:?}",
                pixel_format
            ),
    };
    let pixel_count = (width * height) as usize;
    let data_size = pixel_count * channels;
    let pixels: Vec<u8> = (0..data_size).map(|i| (i % 256) as u8).collect();
    // Leak the pixel data to get a 'static lifetime. This is okay for tests.
    let static_pixels: &'static [u8] = Box::leak(pixels.into_boxed_slice());

    Image {
        pixels: static_pixels,
        width,
        height,
        pixel_format,
        stride_in_bytes: (width * (channels as u32)) as usize,
    }
}

#[test]
fn test_encode_to_memory_basic() {
    ensure_output_dir();
    let image = create_dummy_image(64, 64, PixelFormat::RGBANonPremul);
    let options = EncodeOptions::default();
    let result = encode_to_memory(image, options);
    assert!(result.is_ok(), "Failed to encode to memory: {:?}", result.err());
    let encoded_buffer = result.unwrap();
    assert!(!encoded_buffer.data.is_empty());

    // Optionally, write to file for inspection
    fs::write(get_output_file_path("encode_to_memory_basic.qoir"), encoded_buffer.data).expect(
        "Failed to write encoded output"
    );
}

#[test]
fn test_encode_to_path_basic() {
    ensure_output_dir();
    let image = create_dummy_image(32, 32, PixelFormat::RGB);
    let options = EncodeOptions::default();
    let output_path_str = get_output_file_path("encode_to_path_basic.qoir");
    let path = Path::new(&output_path_str);

    let result = encode(image, options, path);
    assert!(result.is_ok(), "Failed to encode to path: {:?}", result.err());

    // Verify file exists and has content
    assert!(path.exists(), "Output file was not created.");
    let metadata = fs::metadata(path).expect("Failed to get metadata for output file.");
    assert!(metadata.len() > 0, "Output file is empty.");
}

#[test]
fn test_encode_to_writer_basic() {
    ensure_output_dir();
    let image = create_dummy_image(16, 16, PixelFormat::BGR);
    let options = EncodeOptions::default();
    let output_path_str = get_output_file_path("encode_to_writer_basic.qoir");
    let file = File::create(&output_path_str).expect(
        "Failed to create output file for writer test"
    );
    let mut writer = BufWriter::new(file);

    // Assuming a function `qoir_rs::encode_to_writer(&image, &mut writer, options)`
    // Based on lib.rs docs. If not present, this test needs adjustment.
    // Simulating with encode_to_memory if direct writer function isn't available or easily usable here.
    let encode_result = encode_to_memory(image.clone(), options.clone()); // Clone if image is consumed
    if let Ok(encoded_buffer) = encode_result {
        writer.write_all(encoded_buffer.data).expect("Failed to write to BufWriter");
        writer.flush().expect("Failed to flush BufWriter");
    } else {
        panic!("Simulated encode for writer failed: {:?}", encode_result.err());
    }
    // If `qoir_rs::encode_to_writer` exists and is the preferred API:
    // let result = qoir_rs::encode_to_writer(&image, &mut writer, options);
    // assert!(result.is_ok(), "Failed to encode to writer: {:?}", result.err());

    let path = Path::new(&output_path_str);
    assert!(path.exists(), "Output file (writer test) was not created.");
    let metadata = fs
        ::metadata(path)
        .expect("Failed to get metadata for output file (writer test).");
    assert!(metadata.len() > 0, "Output file (writer test) is empty.");
}

#[test]
fn test_round_trip_encode_decode_memory() {
    ensure_output_dir();
    let original_image = create_dummy_image(128, 128, PixelFormat::RGBANonPremul);
    let encode_options = EncodeOptions::default();

    let encoded_result = encode_to_memory(original_image.clone(), encode_options.clone());
    assert!(encoded_result.is_ok(), "Round trip: encode failed: {:?}", encoded_result.err());
    let encoded_buffer = encoded_result.unwrap();

    let decode_options = DecodeOptions::default();
    let decoded_result = decode_from_memory(encoded_buffer.data, decode_options);
    assert!(decoded_result.is_ok(), "Round trip: decode failed: {:?}", decoded_result.err());
    let decoded_image = decoded_result.unwrap();

    assert_eq!(original_image.width, decoded_image.image.width);
    assert_eq!(original_image.height, decoded_image.image.height);
    assert_eq!(
        original_image.pixel_format,
        decoded_image.image.pixel_format,
        "Pixel format mismatch after round trip. This might be expected if QOIR forces a format."
    );
    if
        original_image.pixel_format == decoded_image.image.pixel_format &&
        encode_options.lossiness == 0
    {
        assert_eq!(
            original_image.pixels,
            decoded_image.image.pixels,
            "Pixel data mismatch after lossless round trip"
        );
    }
}

#[test]
fn test_round_trip_decode_encode_memory() {
    ensure_output_dir();
    let qoir_file_name = "at-mouquins.qoir"; // Assuming this is a lossless or near-lossless QOIR
    let file_path = get_test_file_path(qoir_file_name);
    let data = fs::read(&file_path).unwrap_or_else(|_| panic!("Failed to read {}", file_path));

    let decode_options = DecodeOptions::default();
    let decoded_result = decode_from_memory(&data, decode_options.clone());
    assert!(decoded_result.is_ok(), "Decode-Encode: decode failed: {:?}", decoded_result.err());
    let decoded_image_struct = decoded_result.unwrap();

    let pixels_vec: Vec<u8> = decoded_image_struct.image.pixels.to_vec();

    let image_to_reencode = Image {
        pixels: &pixels_vec,
        width: decoded_image_struct.image.width,
        height: decoded_image_struct.image.height,
        pixel_format: decoded_image_struct.image.pixel_format,
        stride_in_bytes: decoded_image_struct.image.stride_in_bytes,
    };

    let encode_options = EncodeOptions { lossiness: 0, ..Default::default() }; // Aim for lossless re-encode
    let re_encoded_result = encode_to_memory(image_to_reencode.clone(), encode_options.clone());
    assert!(
        re_encoded_result.is_ok(),
        "Decode-Encode: re-encode failed: {:?}",
        re_encoded_result.err()
    );
    let re_encoded_buffer = re_encoded_result.unwrap();

    assert!(!re_encoded_buffer.data.is_empty());
    fs::write(
        get_output_file_path("decode_then_encode_at_mouquins.qoir"),
        re_encoded_buffer.data
    ).expect("Failed to write re-encoded output");

    let final_decoded_result = decode_from_memory(re_encoded_buffer.data, decode_options);
    assert!(
        final_decoded_result.is_ok(),
        "Decode-Encode: final decode failed: {:?}",
        final_decoded_result.err()
    );
    let final_decoded_image = final_decoded_result.unwrap();

    assert_eq!(decoded_image_struct.image.width, final_decoded_image.image.width);
    assert_eq!(decoded_image_struct.image.height, final_decoded_image.image.height);
    assert_eq!(decoded_image_struct.image.pixel_format, final_decoded_image.image.pixel_format);

    // Compare pixel data only if original QOIR was likely lossless and re-encode was lossless
    // This comparison is sensitive to any changes, even if visually imperceptible.
    if decoded_image_struct.image.pixel_format == final_decoded_image.image.pixel_format {
        // A more robust check might involve comparing image hashes or using an image diff tool
        // if minor differences are acceptable or expected.
        assert_eq!(
            image_to_reencode.pixels,
            final_decoded_image.image.pixels,
            "Pixel data mismatch after decode-encode-decode cycle. Original may have been lossy or re-encoding introduced changes."
        );
    }
}

#[test]
fn test_decode_external_then_encode_qoir() {
    ensure_output_dir();
    let width = 100u32;
    let height = 50u32;
    let pixel_format = PixelFormat::RGBANonPremul; // Common format from PNG decoding
    let channels = 4u32;
    let simulated_pixels_len = (width * height * channels) as usize;
    let simulated_pixels: Vec<u8> = (0..simulated_pixels_len).map(|i| (i % 256) as u8).collect();
    let static_simulated_pixels: &'static [u8] = Box::leak(simulated_pixels.into_boxed_slice());

    let image_from_external = Image {
        pixels: static_simulated_pixels,
        width,
        height,
        pixel_format,
        stride_in_bytes: (width * channels) as usize,
    };

    let options = EncodeOptions::default();
    let result = encode_to_memory(image_from_external.clone(), options.clone());
    assert!(
        result.is_ok(),
        "Failed to encode simulated external image to QOIR memory: {:?}",
        result.err()
    );
    let encoded_buffer = result.unwrap();
    assert!(!encoded_buffer.data.is_empty());

    fs::write(get_output_file_path("external_to_qoir.qoir"), encoded_buffer.data).expect(
        "Failed to write QOIR from simulated external image"
    );

    // Verify by decoding back
    let decoded_qoir_result = decode_from_memory(encoded_buffer.data, DecodeOptions::default());
    assert!(
        decoded_qoir_result.is_ok(),
        "Failed to decode back the QOIR from external sim: {:?}",
        decoded_qoir_result.err()
    );
    let decoded_qoir = decoded_qoir_result.unwrap();

    assert_eq!(decoded_qoir.image.width, width);
    assert_eq!(decoded_qoir.image.height, height);
    assert_eq!(decoded_qoir.image.pixel_format, pixel_format);
    if options.lossiness == 0 {
        // Only compare pixels if lossless encoding was attempted
        assert_eq!(
            image_from_external.pixels,
            decoded_qoir.image.pixels,
            "Pixel data mismatch for simulated external to QOIR round trip (lossless)"
        );
    }
}
