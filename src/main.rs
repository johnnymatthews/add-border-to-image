use clap::Parser;
use image::{ImageFormat, RgbImage, RgbaImage, DynamicImage, Rgb, Rgba};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Parser)]
#[command(name = "image-border")]
#[command(about = "Add a black border to an image")]
struct Args {
    /// Path to the input image file
    image_path: PathBuf,
}

fn main() {
    let args = Args::parse();
    
    if let Err(e) = process_image(&args.image_path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn process_image(input_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing image: {}", input_path.display());
    
    // Validate input file exists
    if !input_path.exists() {
        return Err("Input file does not exist".into());
    }
    println!("✓ Input file exists");

    // Check if file has supported extension
    let extension = input_path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or("File has no extension")?
        .to_lowercase();

    if !matches!(extension.as_str(), "jpg" | "jpeg" | "png") {
        return Err("Unsupported file format. Only JPG, JPEG, and PNG are supported.".into());
    }
    println!("✓ File format supported: {}", extension.to_uppercase());

    // Load the image
    println!("Loading image...");
    let img = image::open(input_path)
        .map_err(|e| format!("Failed to load image: {}", e))?;
    println!("✓ Image loaded successfully");

    // Calculate border sizes
    let original_width = img.width();
    let original_height = img.height();
    let side_border = (original_width * 5) / 100;  // 5% for left, right, bottom
    let top_border = (original_width * 7) / 100;   // 7% for top

    // Calculate new dimensions
    let new_width = original_width + (2 * side_border);
    let new_height = original_height + top_border + side_border;

    println!("Image dimensions: {}x{}", original_width, original_height);
    println!("Border sizes: top={}px (7% of width), sides={}px (5% of width)", top_border, side_border);
    println!("New dimensions: {}x{}", new_width, new_height);

    // Create new image with border
    println!("Creating bordered image...");
    let bordered_img = match &img {
        DynamicImage::ImageRgba8(_) => {
            let mut new_img = RgbaImage::from_pixel(new_width, new_height, Rgba([0, 0, 0, 255]));
            let rgba_img = img.to_rgba8();
            
            println!("Processing RGBA image with transparency support...");
            
            // Copy original image to center (accounting for different border sizes)
            for y in 0..original_height {
                if y % (original_height / 10).max(1) == 0 {
                    println!("Progress: {}%", (y * 100) / original_height);
                }
                for x in 0..original_width {
                    let pixel = rgba_img.get_pixel(x, y);
                    new_img.put_pixel(x + side_border, y + top_border, *pixel);
                }
            }
            
            DynamicImage::ImageRgba8(new_img)
        }
        _ => {
            let mut new_img = RgbImage::from_pixel(new_width, new_height, Rgb([0, 0, 0]));
            let rgb_img = img.to_rgb8();
            
            println!("Processing RGB image...");
            
            // Copy original image to center (accounting for different border sizes)
            for y in 0..original_height {
                if y % (original_height / 10).max(1) == 0 {
                    println!("Progress: {}%", (y * 100) / original_height);
                }
                for x in 0..original_width {
                    let pixel = rgb_img.get_pixel(x, y);
                    new_img.put_pixel(x + side_border, y + top_border, *pixel);
                }
            }
            
            DynamicImage::ImageRgb8(new_img)
        }
    };
    println!("✓ Border processing complete");

    // Generate output filename
    let output_path = generate_output_path(input_path)?;
    println!("Output file: {}", output_path.display());

    // Determine the original format
    let format = match extension.as_str() {
        "png" => ImageFormat::Png,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        _ => return Err("Unsupported format".into()),
    };

    // Save the image with original format
    println!("Saving image...");
    match format {
        ImageFormat::Png => {
            bordered_img.save_with_format(&output_path, ImageFormat::Png)
                .map_err(|e| format!("Failed to save PNG: {}", e))?;
        }
        ImageFormat::Jpeg => {
            // Save JPEG with high quality to preserve original quality
            let mut output_file = fs::File::create(&output_path)
                .map_err(|e| format!("Failed to create output file: {}", e))?;
            
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, 95);
            let rgb_img = bordered_img.to_rgb8();
            encoder.encode(rgb_img.as_raw(), new_width, new_height, image::ColorType::Rgb8)
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
        }
        _ => unreachable!(),
    }

    println!("✓ Successfully created bordered image: {}", output_path.display());
    Ok(())
}

fn generate_output_path(input_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let parent = input_path.parent().unwrap_or(Path::new("."));
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;
    let extension = input_path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or("Invalid file extension")?;

    let output_filename = format!("{}-with-border.{}", stem, extension);
    Ok(parent.join(output_filename))
}
