use clap::{Parser, Subcommand};
use image::{Rgba, RgbaImage};
use qoir_rs::{
    decode, decode_basic_metadata, decode_from_memory, encode, DecodeOptions, EncodeOptions, Image,
    PixelFormat,
};
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Decode a QOIR file to raw pixels or another format
    Decode {
        /// Input QOIR file
        #[arg(short, long)]
        input: PathBuf,

        /// Output file (use extensions .jpg, .png for conversion)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Pixel format for decoding
        #[arg(short, long, default_value = "rgba")]
        format: String,
    },

    /// Encode an image to QOIR format
    Encode {
        /// Input image file (supported: jpg, png, etc.)
        #[arg(short, long)]
        input: PathBuf,

        /// Output QOIR file
        #[arg(short, long)]
        output: PathBuf,

        /// Lossiness level (0-7, where 0 is lossless)
        #[arg(short, long, default_value = "0")]
        lossiness: u8,

        /// Apply dithering during lossy compression
        #[arg(short, long, default_value = "false")]
        dither: bool,
    },

    /// Display information about a QOIR file
    Info {
        /// QOIR file to inspect
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Convert between image formats
    Convert {
        /// Input image file
        #[arg(short, long)]
        input: PathBuf,

        /// Output image file (use appropriate extension)
        #[arg(short, long)]
        output: PathBuf,

        /// Quality level for JPEG output (1-100)
        #[arg(short, long, default_value = "90")]
        quality: u8,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Decode {
            input,
            output,
            format,
        } => decode_command(input, output, &format)?,
        Commands::Encode {
            input,
            output,
            lossiness,
            dither,
        } => encode_command(input, output, lossiness, dither)?,
        Commands::Info { input } => info_command(input)?,
        Commands::Convert {
            input,
            output,
            quality,
        } => convert_command(input, output, quality)?,
    }

    Ok(())
}

fn decode_command(
    input: PathBuf,
    output: Option<PathBuf>,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse pixel format from string
    let pixel_format = match format.to_lowercase().as_str() {
        "rgba" => PixelFormat::RGBANonPremul,
        "rgb" => PixelFormat::RGB,
        "bgra" => PixelFormat::BGRANonPremul,
        "bgr" => PixelFormat::BGR,
        _ => {
            println!("Unsupported format: {}. Using RGBA.", format);
            PixelFormat::RGBANonPremul
        }
    };

    let options = DecodeOptions {
        pixel_format,
        ..Default::default()
    };

    let decoded = decode(&input, options)?;
    
    println!(
        "Decoded image: {}x{} ({})",
        decoded.image.width, decoded.image.height, format_bytes(decoded.image.pixels.len())
    );
    
    if let Some(output_path) = output {
        let ext = output_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "jpg" | "jpeg" | "png" => {
                // Convert to image crate format and save
                let img = if decoded.image.pixel_format == PixelFormat::RGBANonPremul 
                       || decoded.image.pixel_format == PixelFormat::RGBAPremul {
                    let mut img = RgbaImage::new(decoded.image.width, decoded.image.height);
                    
                    for y in 0..decoded.image.height {
                        for x in 0..decoded.image.width {
                            let idx = (y * decoded.image.stride_in_bytes as u32 + x * 4) as usize;
                            let r = decoded.image.pixels[idx];
                            let g = decoded.image.pixels[idx + 1];
                            let b = decoded.image.pixels[idx + 2];
                            let a = decoded.image.pixels[idx + 3];
                            img.put_pixel(x, y, Rgba([r, g, b, a]));
                        }
                    }
                    
                    image::DynamicImage::ImageRgba8(img)
                } else {
                    // Convert other formats to RGBA
                    return Err("Only RGBA format is currently supported for conversion".into());
                };
                
                match ext.as_str() {
                    "jpg" | "jpeg" => {
                        img.save_with_format(&output_path, image::ImageFormat::Jpeg)?;
                    }
                    "png" => {
                        img.save_with_format(&output_path, image::ImageFormat::Png)?;
                    }
                    _ => unreachable!(),
                }
                
                println!("Image saved to: {}", output_path.display());
            }
            _ => {
                // Save raw pixel data
                let mut file = std::fs::File::create(&output_path)?;
                file.write_all(decoded.image.pixels)?;
                println!("Raw pixel data saved to: {}", output_path.display());
            }
        }
    }

    Ok(())
}

fn encode_command(
    input: PathBuf, 
    output: PathBuf, 
    lossiness: u8,
    dither: bool
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert input image to a format suitable for QOIR encoding
    let img = image::open(&input)?;
    let rgba_img = img.to_rgba8();
    
    let width = rgba_img.width();
    let height = rgba_img.height();
    let pixel_data = rgba_img.into_raw();
    
    let image = Image {
        pixels: &pixel_data,
        width,
        height,
        pixel_format: PixelFormat::RGBANonPremul,
        stride_in_bytes: (width * 4) as usize, // 4 bytes per pixel for RGBA
    };
    
    let options = EncodeOptions {
        lossiness,
        dither,
        ..Default::default()
    };
    
    let encoded = encode(image, options, &output)?;
    
    println!(
        "Image encoded to QOIR: {} ({})", 
        output.display(),
        format_bytes(encoded.data.len())
    );
    
    Ok(())
}

fn info_command(input: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Read QOIR file into memory
    let mut file = File::open(&input)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    // Get basic metadata
    let (width, height, pixel_format) = decode_basic_metadata(&data)?;
    
    println!("QOIR File: {}", input.display());
    println!("Dimensions: {}x{}", width, height);
    println!("Pixel Format: {:?}", pixel_format);
    println!("File Size: {}", format_bytes(data.len()));
    
    // Get more detailed information if possible
    match decode_from_memory(&data, DecodeOptions::default()) {
        Ok(decoded) => {
            println!("Decoded Image Size: {}", format_bytes(decoded.image.pixels.len()));
            
            if decoded.cic_profile.is_some() {
                println!("Has CIC Profile: Yes");
            }
            if decoded.icc_profile.is_some() {
                println!("Has ICC Profile: Yes");
            }
            if decoded.exif.is_some() {
                println!("Has EXIF Data: Yes");
            }
            if decoded.xmp.is_some() {
                println!("Has XMP Data: Yes");
            }
        }
        Err(e) => {
            println!("Warning: Could not fully decode image: {:?}", e);
        }
    }
    
    Ok(())
}

fn convert_command(
    input: PathBuf,
    output: PathBuf, 
    quality: u8
) -> Result<(), Box<dyn std::error::Error>> {
    let in_ext = input.extension().and_then(|e| e.to_str()).unwrap_or("");
    let out_ext = output.extension().and_then(|e| e.to_str()).unwrap_or("");
    
    if in_ext.eq_ignore_ascii_case("qoir") {
        // QOIR to other format
        let decoded = decode(&input, DecodeOptions::default())?;
        
        // Convert to image crate format
        if decoded.image.pixel_format == PixelFormat::RGBANonPremul 
           || decoded.image.pixel_format == PixelFormat::RGBAPremul {
            let mut img = RgbaImage::new(decoded.image.width, decoded.image.height);
            
            for y in 0..decoded.image.height {
                for x in 0..decoded.image.width {
                    let idx = (y * decoded.image.stride_in_bytes as u32 + x * 4) as usize;
                    let r = decoded.image.pixels[idx];
                    let g = decoded.image.pixels[idx + 1];
                    let b = decoded.image.pixels[idx + 2];
                    let a = decoded.image.pixels[idx + 3];
                    img.put_pixel(x, y, Rgba([r, g, b, a]));
                }
            }
            
            match out_ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => {
                    image::DynamicImage::ImageRgba8(img)
                        .save_with_format(&output, image::ImageFormat::Jpeg)?;
                }
                "png" => {
                    image::DynamicImage::ImageRgba8(img)
                        .save_with_format(&output, image::ImageFormat::Png)?;
                }
                _ => {
                    return Err(format!("Unsupported output format: {}", out_ext).into());
                }
            }
        } else {
            return Err("Only RGBA format is currently supported for conversion".into());
        }
    } else if out_ext.eq_ignore_ascii_case("qoir") {
        // Other format to QOIR
        let img = image::open(&input)?;
        let rgba_img = img.to_rgba8();
        
        let width = rgba_img.width();
        let height = rgba_img.height();
        let pixel_data = rgba_img.into_raw();
        
        let image = Image {
            pixels: &pixel_data,
            width,
            height,
            pixel_format: PixelFormat::RGBANonPremul,
            stride_in_bytes: (width * 4) as usize,
        };
        
        encode(image, EncodeOptions {
            lossiness: quality,
            ..Default::default()
        }, &output)?;
    } else {
        // Convert between non-QOIR formats using the image crate
        let img = image::open(&input)?;
        
        match out_ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => {
                img.save_with_format(&output, image::ImageFormat::Jpeg)?;
            }
            "png" => {
                img.save_with_format(&output, image::ImageFormat::Png)?;
            }
            _ => {
                return Err(format!("Unsupported output format: {}", out_ext).into());
            }
        }
    }
    
    println!("Converted {} to {}", input.display(), output.display());
    Ok(())
}

// Helper function to format byte sizes in a human-readable way
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    
    if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
