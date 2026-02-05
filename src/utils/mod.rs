// pixie/src/utils/mod.rs
use crate::core::{ImageToolError, Result};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn generate_output_path(
    input_path: &Path,
    output: Option<&Path>,
    suffix: &str,
) -> PathBuf {
    match output {
        Some(path) => path.to_path_buf(),
        None => {
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image");
            let extension = input_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("jpg");

            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let mut new_filename = format!("{}_{}_{}.{}", stem, suffix, timestamp, extension);
            let mut counter = 1;

            // Ensure we don't overwrite existing files
            while Path::new(&new_filename).exists() {
                new_filename = format!("{}_{}_{}_{}.{}", stem, suffix, timestamp, counter, extension);
                counter += 1;
            }

            input_path.with_file_name(new_filename)
        }
    }
}

pub fn format_file_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let base = 1024_f64;
    let bytes_f64 = bytes as f64;
    let exponent = (bytes_f64.log10() / base.log10()).floor() as i32;
    let size = bytes_f64 / base.powi(exponent);

    format!("{:.2} {}", size, UNITS[exponent as usize])
}

pub fn calculate_aspect_ratio(width: u32, height: u32) -> f32 {
    if height == 0 {
        0.0
    } else {
        width as f32 / height as f32
    }
}

pub fn validate_dimensions(width: u32, height: u32) -> Result<()> {
    if width > 100_000 || height > 100_000 {
        return Err(ImageToolError::InvalidParameter(
            "Dimensions too large (max 100,000 pixels)".to_string()
        ));
    }

    if width == 0 && height == 0 {
        return Err(ImageToolError::InvalidParameter(
            "At least one dimension must be specified".to_string()
        ));
    }

    Ok(())
}

pub fn get_image_info(path: &Path) -> Result<(u32, u32, String)> {
    let file = std::fs::File::open(path)?;
    let reader = image::io::Reader::new(std::io::BufReader::new(file))
        .with_guessed_format()?;

    let format = reader.format()
        .map(|f| f.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let dimensions = reader.into_dimensions()?;

    Ok((dimensions.0, dimensions.1, format))
}

pub fn is_supported_format(path: &Path) -> bool {
    let extensions = [
        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp",
    ];

    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    filename
        .chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect()
}

pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}