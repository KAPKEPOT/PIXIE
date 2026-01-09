use crate::{ImageToolError, Result};
use image::{DynamicImage, ImageFormat, ImageOutputFormat};
use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::Path;

pub struct ImageCompressor {
    quality: u8,
}

impl ImageCompressor {
    pub fn new(quality: u8) -> Self {
        Self { quality: quality.clamp(1, 100) }
    }
    
    pub fn save(&self, image: &DynamicImage, path: &Path) -> Result<()> {
        let format = self.detect_format(path);
        
        log::debug!(
            "Saving image to {} with format {:?}, quality: {}",
            path.display(),
            format,
            self.quality
        );
        
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        
        match format {
            ImageFormat::Jpeg => {
                image.write_to(writer, ImageOutputFormat::Jpeg(self.quality))?;
            }
            ImageFormat::Png => {
                image.write_to(writer, ImageOutputFormat::Png)?;
            }
            ImageFormat::WebP => {
                // Note: WebP support might require additional features
                image.write_to(writer, ImageOutputFormat::Unsupported("webp".to_string()))?;
            }
            _ => {
                // For other formats, use default settings
                image.write_to(writer, ImageOutputFormat::from(format))?;
            }
        }
        
        let file_size = std::fs::metadata(path)?.len();
        log::info!("Saved image: {} ({} bytes)", path.display(), file_size);
        
        Ok(())
    }
    
    pub fn compress_to_bytes(&self, image: &DynamicImage, format: ImageFormat) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        
        match format {
            ImageFormat::Jpeg => {
                image.write_to(&mut buffer, ImageOutputFormat::Jpeg(self.quality))?;
            }
            ImageFormat::Png => {
                image.write_to(&mut buffer, ImageOutputFormat::Png)?;
            }
            _ => {
                image.write_to(&mut buffer, ImageOutputFormat::from(format))?;
            }
        }
        
        Ok(buffer.into_inner())
    }
    
    pub fn optimize_jpeg(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Reload and re-save with new quality
        let image = image::load_from_memory(data)
            .map_err(|e| ImageToolError::ProcessingError(format!("Failed to load JPEG: {}", e)))?;
        
        self.compress_to_bytes(&image, ImageFormat::Jpeg)
    }
    
    pub fn optimize_png(&self, data: &[u8]) -> Result<Vec<u8>> {
        // For PNG, we could use oxipng for better optimization
        // For now, just return the original
        Ok(data.to_vec())
    }
    
    fn detect_format(&self, path: &Path) -> ImageFormat {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
            Some("png") => ImageFormat::Png,
            Some("gif") => ImageFormat::Gif,
            Some("bmp") => ImageFormat::Bmp,
            Some("webp") => ImageFormat::WebP,
            Some("tiff") | Some("tif") => ImageFormat::Tiff,
            _ => ImageFormat::Jpeg, // default to JPEG
        }
    }
    
    pub fn calculate_savings(&self, original_size: u64, compressed_size: u64) -> f64 {
        if original_size == 0 {
            return 0.0;
        }
        
        let savings = (original_size - compressed_size) as f64 / original_size as f64 * 100.0;
        savings.max(0.0)
    }
}