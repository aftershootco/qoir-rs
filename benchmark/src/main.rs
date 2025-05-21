#![allow(clippy::type_complexity)]

use clap::Parser;
use image::{ ColorType, ImageEncoder, ImageFormat };
use qoir_rs::{
    decode_from_memory,
    encode_to_memory,
    DecodeOptions,
    EncodeOptions,
    Image as QoirImage,
    PixelFormat,
};
use std::{ fs, path::{ Path, PathBuf }, time::{ Duration, Instant } };
use tempfile::TempDir;

#[derive(Parser, Debug)]
#[command(author, version, about = "Benchmark image format performance")]
struct Args {
    /// Input directory containing source images
    #[arg(help = "Path to directory containing source images")]
    input_dir: PathBuf,

    /// Number of iterations per image
    #[arg(short, long, default_value = "100")]
    iterations: usize,

    /// Frequency of progress updates
    #[arg(short, long, default_value = "10")]
    freq: usize,
}

// Common image data structure that works across different libraries
struct ImageData {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    bytes_per_pixel: usize,
}

// A trait for image encoders to be benchmarked
trait BenchmarkEncoder {
    fn name(&self) -> &str;
    fn encode(&self, image: &ImageData) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

// A trait for image decoders to be benchmarked
trait BenchmarkDecoder {
    fn name(&self) -> &str;
    fn decode(&self, data: &[u8]) -> Result<ImageData, Box<dyn std::error::Error>>;
}

// Implementation for QOIR encoder
struct QoirEncoder {
    options: EncodeOptions,
}

impl BenchmarkEncoder for QoirEncoder {
    fn name(&self) -> &str {
        "QOIR"
    }

    fn encode(&self, image: &ImageData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let qoir_image = QoirImage {
            pixels: &image.pixels,
            width: image.width,
            height: image.height,
            pixel_format: PixelFormat::RGBANonPremul,
            stride_in_bytes: (image.width as usize) * image.bytes_per_pixel,
        };

        let encoded = encode_to_memory(qoir_image, self.options.clone())?;
        Ok(encoded.data.to_vec())
    }
}

// Implementation for QOIR decoder
struct QoirDecoder {
    options: DecodeOptions,
}

impl BenchmarkDecoder for QoirDecoder {
    fn name(&self) -> &str {
        "QOIR"
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData, Box<dyn std::error::Error>> {
        let decoded = decode_from_memory(data, self.options.clone())?;

        Ok(ImageData {
            pixels: decoded.image.pixels.to_vec(),
            width: decoded.image.width,
            height: decoded.image.height,
            bytes_per_pixel: match decoded.image.pixel_format {
                PixelFormat::RGB => 3,
                PixelFormat::BGR => 3,
                PixelFormat::RGBANonPremul | PixelFormat::RGBAPremul => 4,
                PixelFormat::BGRANonPremul | PixelFormat::BGRAPremul => 4,
                PixelFormat::RGBX => 4,
                PixelFormat::BGRX => 4,
                _ => 4,
            },
        })
    }
}

// Implementation for JPEG encoder using image crate
struct JpegEncoder {
    quality: u8,
}

impl BenchmarkEncoder for JpegEncoder {
    fn name(&self) -> &str {
        "JPEG"
    }

    fn encode(&self, image: &ImageData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output = Vec::new();
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, self.quality);
        encoder.write_image(&image.pixels, image.width, image.height, ColorType::Rgba8)?;
        Ok(output)
    }
}

// Implementation for JPEG decoder using image crate
struct JpegDecoder;

impl BenchmarkDecoder for JpegDecoder {
    fn name(&self) -> &str {
        "JPEG"
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData, Box<dyn std::error::Error>> {
        let img = image::load_from_memory_with_format(data, ImageFormat::Jpeg)?;
        let rgba = img.to_rgba8();

        Ok(ImageData {
            pixels: rgba.into_raw(),
            width: img.width(),
            height: img.height(),
            bytes_per_pixel: 4,
        })
    }
}

// Implementation for PNG encoder using image crate
struct PngEncoder;

impl BenchmarkEncoder for PngEncoder {
    fn name(&self) -> &str {
        "PNG"
    }

    fn encode(&self, image: &ImageData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut output);
        encoder.write_image(&image.pixels, image.width, image.height, ColorType::Rgba8)?;
        Ok(output)
    }
}

// Implementation for PNG decoder using image crate
struct PngDecoder;

impl BenchmarkDecoder for PngDecoder {
    fn name(&self) -> &str {
        "PNG"
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData, Box<dyn std::error::Error>> {
        let img = image::load_from_memory_with_format(data, ImageFormat::Png)?;
        let rgba = img.to_rgba8();

        Ok(ImageData {
            pixels: rgba.into_raw(),
            width: img.width(),
            height: img.height(),
            bytes_per_pixel: 4,
        })
    }
}

#[derive(Debug)]
struct BenchmarkResults {
    encoder_name: String,
    num_images_tested: usize,
    #[allow(unused)]
    num_iterations_per_image: usize,
    avg_time_per_image_ms: f64,
    total_time_s: f64,
    avg_size_original_kb: f64,
    avg_size_processed_kb: f64,
    avg_size_change_percentage: f64,
    throughput_mb_s: f64,
    speed_images_s: f64,
}

impl std::fmt::Display for BenchmarkResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "| {:<10} | {:<6} | {:<8.2} | {:<8.2} | {:<10.2} | {:<10.2} | {:<6.2} | {:<8.2} | {:<8.2} |",
            self.encoder_name,
            self.num_images_tested,
            self.avg_time_per_image_ms,
            self.total_time_s,
            self.avg_size_original_kb,
            self.avg_size_processed_kb,
            self.avg_size_change_percentage,
            self.throughput_mb_s,
            self.speed_images_s
        )
    }
}

// Image file conversion utilities
struct ConvertedImages {
    #[allow(unused)]
    temp_dir: TempDir,
    png_files: Vec<(Vec<u8>, usize)>,
    jpeg_files: Vec<(Vec<u8>, usize)>,
    qoir_files: Vec<(Vec<u8>, usize)>,
    rgba_images: Vec<ImageData>,
}

fn prepare_images(input_dir: &Path) -> Result<ConvertedImages, Box<dyn std::error::Error>> {
    println!("Scanning for images in: {}", input_dir.display());

    // Create temporary directory
    let temp_dir = TempDir::new()?;
    println!("Created temporary directory at: {}", temp_dir.path().display());

    // Scan the input directory for image files
    let mut source_images = Vec::new();
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process files with image extensions
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ["jpg", "jpeg", "png", "gif", "bmp"].contains(&ext.as_str()) {
                    match image::open(&path) {
                        Ok(img) => {
                            println!("Found image: {}", path.display());
                            source_images.push((
                                path.file_name().unwrap().to_string_lossy().to_string(),
                                img,
                            ));
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to open {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
    }

    if source_images.is_empty() {
        return Err("No valid images found in the input directory".into());
    }

    println!("Found {} images to convert", source_images.len());

    // Generate test images in all formats
    let mut png_files = Vec::new();
    let mut jpeg_files = Vec::new();
    let mut qoir_files = Vec::new();
    let mut rgba_images = Vec::new();

    for (filename, img) in source_images {
        // Save as RGBA for memory testing
        let rgba = img.to_rgba8();
        rgba_images.push(ImageData {
            pixels: rgba.clone().into_raw(),
            width: rgba.width(),
            height: rgba.height(),
            bytes_per_pixel: 4,
        });

        // Save as PNG
        let png_path = temp_dir.path().join(format!("{}.png", filename));
        img.save_with_format(&png_path, ImageFormat::Png)?;
        let png_buffer = fs::read(&png_path)?;
        let png_size = png_buffer.len();
        png_files.push((png_buffer, png_size));

        // Save as JPEG
        let jpeg_path = temp_dir.path().join(format!("{}.jpg", filename));
        img.save_with_format(&jpeg_path, ImageFormat::Jpeg)?;
        let jpeg_buffer = fs::read(&jpeg_path)?;
        let jpeg_size = jpeg_buffer.len();
        jpeg_files.push((jpeg_buffer, jpeg_size));

        // Save as QOIR
        let qoir_path = temp_dir.path().join(format!("{}.qoir", filename));
        let width = rgba.width();
        let height = rgba.height();
        let qoir_image = QoirImage {
            pixels: &rgba.into_raw(),
            width,
            height,
            pixel_format: PixelFormat::RGBANonPremul,
            stride_in_bytes: (width as usize) * 4,
        };

        let qoir_options = EncodeOptions {
            lossiness: 0,
            dither: false,
            ..Default::default()
        };

        let encoded_qoir = encode_to_memory(qoir_image, qoir_options)?;
        let qoir_buffer = encoded_qoir.data.to_vec();
        let qoir_size = qoir_buffer.len();
        fs::write(&qoir_path, &qoir_buffer)?;
        qoir_files.push((qoir_buffer, qoir_size));
    }

    println!("Converted all images to PNG, JPEG, and QOIR formats");

    Ok(ConvertedImages {
        temp_dir,
        png_files,
        jpeg_files,
        qoir_files,
        rgba_images,
    })
}

fn benchmark_encode<E: BenchmarkEncoder>(
    encoder: &E,
    images: &[ImageData],
    iterations: usize,
    freq: usize,
) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
    println!("Running {} Encode Benchmark...", encoder.name());

    let mut total_encoding_time = Duration::new(0, 0);
    let mut total_input_pixel_bytes_processed: usize = 0;
    let mut total_output_bytes_processed: usize = 0;
    let mut encoding_times_ms: Vec<f64> = Vec::new();

    for iter in 0..iterations {
        if iter % freq == 0 {
            println!("Processing batch {}/{}", iter + 1, iterations);
        }
        for image in images {
            let input_size = image.pixels.len();
            total_input_pixel_bytes_processed += input_size;

            let start_time = Instant::now();
            let encoded_data = encoder.encode(image)?;
            let duration = start_time.elapsed();

            total_encoding_time += duration;
            encoding_times_ms.push(duration.as_secs_f64() * 1000.0);
            total_output_bytes_processed += encoded_data.len();
        }
    }

    let num_images_tested = images.len();
    let total_operations = num_images_tested * iterations;

    let avg_time_per_image_ms = if !encoding_times_ms.is_empty() {
        encoding_times_ms.iter().sum::<f64>() / (encoding_times_ms.len() as f64)
    } else {
        0.0
    };
    let total_time_s = total_encoding_time.as_secs_f64();

    let avg_size_original_kb = if num_images_tested > 0 {
        (total_input_pixel_bytes_processed as f64) /
            (iterations as f64) /
            (num_images_tested as f64) /
            1024.0
    } else {
        0.0
    };

    let avg_size_processed_kb = if total_operations > 0 {
        (total_output_bytes_processed as f64) / (total_operations as f64) / 1024.0
    } else {
        0.0
    };

    let avg_size_change_percentage = if avg_size_original_kb > 0.0 {
        (avg_size_processed_kb / avg_size_original_kb) * 100.0
    } else {
        0.0
    };

    let throughput_mb_s = if total_time_s > 0.0 {
        (total_input_pixel_bytes_processed as f64) / (1024.0 * 1024.0) / total_time_s
    } else {
        0.0
    };

    let speed_images_s = if total_time_s > 0.0 {
        (total_operations as f64) / total_time_s
    } else {
        0.0
    };

    Ok(BenchmarkResults {
        encoder_name: encoder.name().to_string(),
        num_images_tested,
        num_iterations_per_image: iterations,
        avg_time_per_image_ms,
        total_time_s,
        avg_size_original_kb,
        avg_size_processed_kb,
        avg_size_change_percentage,
        throughput_mb_s,
        speed_images_s,
    })
}

fn benchmark_decode<D: BenchmarkDecoder>(
    decoder: &D,
    files: &[(Vec<u8>, usize)],
    iterations: usize,
    freq: usize,
) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
    println!("Running {} Decode Benchmark...", decoder.name());

    let mut total_decoding_time = Duration::new(0, 0);
    let mut total_input_bytes_processed: usize = 0;
    let mut total_output_pixel_bytes_processed: usize = 0;
    let mut decoding_times_ms: Vec<f64> = Vec::new();

    for iter in 0..iterations {
        if iter % freq == 0 {
            println!("Processing batch {}/{}", iter + 1, iterations);
        }
        for (buffer, original_size) in files {
            total_input_bytes_processed += original_size;

            let start_time = Instant::now();
            let decoded_image = decoder.decode(buffer)?;
            let duration = start_time.elapsed();

            total_decoding_time += duration;
            decoding_times_ms.push(duration.as_secs_f64() * 1000.0);
            total_output_pixel_bytes_processed += decoded_image.pixels.len();
        }
    }

    let num_files_tested = files.len();
    let total_operations = num_files_tested * iterations;

    let avg_time_per_file_ms = if !decoding_times_ms.is_empty() {
        decoding_times_ms.iter().sum::<f64>() / (decoding_times_ms.len() as f64)
    } else {
        0.0
    };
    let total_time_s = total_decoding_time.as_secs_f64();

    let avg_size_original_kb = if num_files_tested > 0 {
        (total_input_bytes_processed as f64) /
            (iterations as f64) /
            (num_files_tested as f64) /
            1024.0
    } else {
        0.0
    };

    let avg_size_processed_kb = if total_operations > 0 {
        (total_output_pixel_bytes_processed as f64) / (total_operations as f64) / 1024.0
    } else {
        0.0
    };

    let avg_size_change_percentage = if avg_size_original_kb > 0.0 {
        (avg_size_processed_kb / avg_size_original_kb) * 100.0
    } else {
        0.0
    };

    let throughput_mb_s = if total_time_s > 0.0 {
        (total_input_bytes_processed as f64) / (1024.0 * 1024.0) / total_time_s
    } else {
        0.0
    };

    let speed_images_s = if total_time_s > 0.0 {
        (total_operations as f64) / total_time_s
    } else {
        0.0
    };

    Ok(BenchmarkResults {
        encoder_name: decoder.name().to_string(),
        num_images_tested: num_files_tested,
        num_iterations_per_image: iterations,
        avg_time_per_image_ms: avg_time_per_file_ms,
        total_time_s,
        avg_size_original_kb,
        avg_size_processed_kb,
        avg_size_change_percentage,
        throughput_mb_s,
        speed_images_s,
    })
}

fn print_benchmark_table_header(title: &str) {
    println!("\n{}", title);
    println!(
        "|-----------+--------+----------+----------+------------+------------+--------+----------+------------|"
    );
    println!(
        "| Format    | Images | Avg Time | Total    | Orig Size  | Proc Size  | Size   | Thrghpt  | Speed      |"
    );
    println!(
        "|           |        | (ms)     | Time (s) | (KB)       | (KB)       | (%)    | (MB/s)   | (imgs/s)   |"
    );
    println!(
        "|-----------+--------+----------+----------+------------+------------+--------+----------+------------|"
    );
}

fn print_benchmark_table_footer() {
    println!(
        "|-----------+--------+----------+----------+------------+------------+--------+----------+------------|"
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();
    let iterations = args.iterations;
    let freq = args.freq;

    println!("Starting Image Format Benchmark ({} iterations per image)...", iterations);
    println!("Using images from: {}", args.input_dir.display());

    // Prepare test images
    let converted_images = match prepare_images(&args.input_dir) {
        Ok(images) => images,
        Err(e) => {
            eprintln!("Failed to prepare test images: {}", e);
            return Err(e);
        }
    };

    // Create encoders
    let qoir_encoder = QoirEncoder {
        options: EncodeOptions {
            lossiness: 0, // Lossless
            dither: false,
            ..Default::default()
        },
    };

    let jpeg_encoder = JpegEncoder { quality: 90 };
    let png_encoder = PngEncoder;

    // Create decoders
    let qoir_decoder = QoirDecoder {
        options: DecodeOptions::default(),
    };

    let jpeg_decoder = JpegDecoder;
    let png_decoder = PngDecoder;

    // Run encoding benchmarks
    let mut encode_results = Vec::new();

    if let Ok(results) = benchmark_encode(&qoir_encoder, &converted_images.rgba_images, iterations, freq) {
        encode_results.push(results);
    }

    if let Ok(results) = benchmark_encode(&jpeg_encoder, &converted_images.rgba_images, iterations, freq) {
        encode_results.push(results);
    }

    if let Ok(results) = benchmark_encode(&png_encoder, &converted_images.rgba_images, iterations, freq) {
        encode_results.push(results);
    }

    // Display encoding results
    print_benchmark_table_header("ENCODING BENCHMARK RESULTS");
    for result in &encode_results {
        println!("{}", result);
    }
    print_benchmark_table_footer();

    // Run decoding benchmarks
    let mut decode_results = Vec::new();

    // QOIR decoding benchmark
    if !converted_images.qoir_files.is_empty() {
        if
            let Ok(results) = benchmark_decode(
                &qoir_decoder,
                &converted_images.qoir_files,
                iterations,
                freq
            )
        {
            decode_results.push(results);
        }
    } else {
        eprintln!("Warning: No QOIR files available for decoding benchmark");
    }

    // JPEG decoding benchmark
    if !converted_images.jpeg_files.is_empty() {
        if
            let Ok(results) = benchmark_decode(
                &jpeg_decoder,
                &converted_images.jpeg_files,
                iterations,
                freq
            )
        {
            decode_results.push(results);
        }
    } else {
        eprintln!("Warning: No JPEG files available for decoding benchmark");
    }

    // PNG decoding benchmark
    if !converted_images.png_files.is_empty() {
        if
            let Ok(results) = benchmark_decode(
                &png_decoder,
                &converted_images.png_files,
                iterations,
                freq
            )
        {
            decode_results.push(results);
        }
    } else {
        eprintln!("Warning: No PNG files available for decoding benchmark");
    }

    // Display decoding results
    print_benchmark_table_header("DECODING BENCHMARK RESULTS");
    for result in &decode_results {
        println!("{}", result);
    }
    print_benchmark_table_footer();

    println!("\nBenchmarks finished.");
    // Temp dir will be automatically cleaned up when the ConvertedImages struct goes out of scope

    Ok(())
}
