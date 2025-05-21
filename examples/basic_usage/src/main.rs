use qoir_rs::{decode, encode, DecodeOptions, EncodeOptions, Error, Image, PixelFormat};
use std::path::Path;

fn main() -> Result<(), Error> {
    // --- Decoding Example ---
    println!("Decoding example image...");
    let decode_options = DecodeOptions::default();
    // Assuming at-mouquins.qoir is in the root of the qoir-rs project, not inside basic_usage
    let input_path_str = "harvesters.qoir"; 
    let input_path = Path::new(input_path_str);

    if !input_path.exists() {
        eprintln!("Error: Input file not found at {}", input_path.display());
        eprintln!("Please ensure 'harvesters.qoir' is in the root of the 'qoir-rs' project.");
        return Err(Error::FileNotFound);
    }

    match decode(input_path_str, decode_options) {
        Ok(decoded_image) => {
            println!(
                "Image '{}' decoded successfully: {}x{}",
                input_path.display(), decoded_image.image.width, decoded_image.image.height
            );
            println!("Pixel format: {:?}", decoded_image.image.pixel_format);
            println!("Stride in bytes: {}", decoded_image.image.stride_in_bytes);

            // You can access pixel data like this:
            // let pixels = decoded_image.image.pixels;
            // if !pixels.is_empty() {
            //     println!("First 10 bytes of pixel data: {:?}", &pixels[0..10.min(pixels.len())]);
            // }
        }
        Err(e) => {
            eprintln!("Error decoding image '{}': {:?}", input_path.display(), e);
            return Err(e); 
        }
    }

    println!("\n--- Encoding Example ---");
    let width = 32;
    let height = 32;
    let mut pixel_data = vec![0u8; (width * height * 4) as usize]; // RGBA

    for y in 0..height {
        for x in 0..width {
            let offset = ((y * width + x) * 4) as usize;
            pixel_data[offset] = (x * 255 / width) as u8; // Red gradient
            pixel_data[offset + 1] = (y * 255 / height) as u8; // Green gradient
            pixel_data[offset + 2] = 128; // Blue
            pixel_data[offset + 3] = 255; // Alpha
        }
    }

    let image_to_encode = Image {
        pixels: &pixel_data,
        width,
        height,
        pixel_format: PixelFormat::RGBANonPremul,
        stride_in_bytes: (width * 4) as usize,
    };

    let encode_options = EncodeOptions {
        lossiness: 0, // Lossless
        ..Default::default()
    };

    
    // Output path will be relative to the execution directory of basic_usage,
    // which is examples/basic_usage/target/debug or release
    // For simplicity, let's try to put it in the qoir-rs root for now.
    let output_path_str = "./output/encoded_example.qoir";
    let output_path = Path::new(output_path_str);
    if !output_path.parent().unwrap().exists() {
        std::fs::create_dir_all(output_path.parent().unwrap()).expect("Failed to create output directory");
    }

    println!("Encoding dummy image to '{}'...", output_path.display());

    match encode(image_to_encode, encode_options, output_path_str) {
        Ok(_) => {
            println!("Image encoded and saved to '{}' successfully.", output_path.display());
        }
        Err(e) => {
            eprintln!("Error encoding image to '{}': {:?}", output_path.display(), e);
            return Err(e);
        }
    }

    Ok(())
}

