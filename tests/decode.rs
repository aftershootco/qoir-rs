use qoir_rs::{decode, decode_from_memory, decode_from_reader, DecodeOptions};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

const TEST_DATA_DIR: &str = "tests/data";
const TEST_OUTPUT_DIR: &str = "tests/output";

fn ensure_output_dir() {
    fs::create_dir_all(TEST_OUTPUT_DIR).expect("Failed to create output directory");
}

fn get_test_file_path(name: &str) -> String {
    format!("{}/{}", TEST_DATA_DIR, name)
}

#[test]
fn test_decode_from_memory_valid_qoir() {
    let test_files = [
        "at-mouquins.qoir",
        "at-mouquins.lossy-naive-dither-2.qoir",
        "hibiscus.regular.qoir",
    ];
    let options = DecodeOptions::default();

    for file_name in test_files.iter() {
        let file_path = get_test_file_path(file_name);
        let data = fs::read(&file_path).unwrap_or_else(|_| panic!("Failed to read {}", file_path));
        let result = decode_from_memory(&data, options.clone());
        assert!(result.is_ok(), "Failed to decode {} from memory: {:?}", file_name, result.err());
        let decoded_image = result.unwrap();

        // Basic checks - specific values depend on the image content
        assert!(decoded_image.image.width > 0);
        assert!(decoded_image.image.height > 0);
        assert!(!decoded_image.image.pixels.is_empty());
    }
}

#[test]
fn test_decode_from_path_valid_qoir() {
    let test_files = [
        "at-mouquins.qoir",
        "at-mouquins.lossy-naive-dither-2.qoir",
        "hibiscus.regular.qoir",
    ];
    let options = DecodeOptions::default();

    for file_name in test_files.iter() {
        let file_path_str = get_test_file_path(file_name);
        let path = Path::new(&file_path_str);
        let result = decode(path, options.clone());
        assert!(result.is_ok(), "Failed to decode {} from path: {:?}", file_name, result.err());
        let decoded_image = result.unwrap();

        assert!(decoded_image.image.width > 0);
        assert!(decoded_image.image.height > 0);
        assert!(!decoded_image.image.pixels.is_empty());
    }
}

#[test]
fn test_decode_from_reader_valid_qoir() {
    let test_files = ["at-mouquins.qoir", "harvesters.qoir"];
    let options = DecodeOptions::default();

    for file_name in test_files.iter() {
        let file_path_str = get_test_file_path(file_name);
        let file = File::open(&file_path_str).unwrap_or_else(|_| panic!("Failed to open {}", file_path_str));
        let reader = BufReader::new(file);
        let result = decode_from_reader(reader, options.clone()); // Simulate if no direct reader fn
        assert!(result.is_ok(), "Failed to decode {} via reader: {:?}", file_name, result.err());
        let decoded_image = result.unwrap();
        assert!(decoded_image.image.width > 0);
        assert!(decoded_image.image.height > 0);
    }
}

#[test]
fn test_decode_from_memory_invalid_data() {
    let invalid_data: &[u8] = &[0, 1, 2, 3, 4, 5];
    let options = DecodeOptions::default();
    let result = decode_from_memory(invalid_data, options);
    assert!(result.is_err(), "Decoding invalid data should fail");
}

#[test]
fn test_decode_from_path_non_existent_file() {
    ensure_output_dir(); // Ensure parent dir for path exists if needed by OS
    let path = Path::new("tests/data/non_existent_file.qoir");
    let options = DecodeOptions::default();
    let result = decode(path, options);
    assert!(result.is_err(), "Decoding non-existent file should fail");
}
