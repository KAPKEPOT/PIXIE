use crate::{ImageToolError, Result};
use image::{DynamicImage, ImageReader};
use std::path::Path;

pub struct ImageLoader;

impl ImageLoader {
    pub fn new() -> Self {
        Self
    }
    
    pub fn load(&self, path: &Path) -> Result<DynamicImage> {
        log::debug!("Loading image from: {}", path.display());
        
        if !path.exists() {
            return Err(ImageToolError::InvalidParameter(
                format!("File does not exist: {}", path.display())
            ));
        }
        
        let image = ImageReader::open(path)?
            .with_guessed_format()?
            .decode()
            .map_err(|e| {
                ImageToolError::ProcessingError(format!("Failed to decode image: {}", e))
            })?;
        
        let (width, height) = image.dimensions();
        let format = image.color();
        
        log::info!(
            "Loaded image: {}x{} pixels, format: {:?}",
            width, height, format
        );
        
        Ok(image)
    }
    
    pub fn load_from_bytes(&self, data: &[u8]) -> Result<DynamicImage> {
        let image = image::load_from_memory(data)
            .map_err(|e| {
                ImageToolError::ProcessingError(format!("Failed to decode image from bytes: {}", e))
            })?;
        
        Ok(image)
    }
}