pub mod cli;
pub mod loader;
pub mod resizer;
pub mod compressor;
pub mod metadata;
pub mod batch;
pub mod utils;

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum ResizeAlgorithm {
    Nearest,
    Bilinear,
    Bicubic,
    Lanczos3,
}

#[derive(Debug, Clone)]
pub struct ProcessConfig {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub quality: u8,
    pub keep_aspect: bool,
    pub strip_metadata: bool,
    pub algorithm: ResizeAlgorithm,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            scale: 0.0,
            quality: 85,
            keep_aspect: true,
            strip_metadata: false,
            algorithm: ResizeAlgorithm::Lanczos3,
        }
    }
}

#[derive(Error, Debug)]
pub enum ImageToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

pub type Result<T> = std::result::Result<T, ImageToolError>;

pub struct ImageProcessor {
    config: ProcessConfig,
}

impl ImageProcessor {
    pub fn new(config: ProcessConfig) -> Self {
        Self { config }
    }
    
    pub fn process<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        use loader::ImageLoader;
        use resizer::ImageResizer;
        use compressor::ImageCompressor;
        use metadata::MetadataStripper;
        
        let loader = ImageLoader::new();
        let mut image = loader.load(input_path.as_ref())?;
        
        // Strip metadata if requested
        if self.config.strip_metadata {
            let stripper = MetadataStripper::new();
            stripper.strip_metadata(&mut image)?;
        }
        
        // Resize if needed
        if self.config.width > 0 || self.config.height > 0 || self.config.scale > 0.0 {
            let resizer = ImageResizer::new(self.config.algorithm, self.config.keep_aspect);
            
            let mode = if self.config.scale > 0.0 {
                resizer::ResizeMode::Scale(self.config.scale)
            } else {
                resizer::ResizeMode::Absolute(self.config.width, self.config.height)
            };
            
            image = resizer.resize(&image, mode);
        }
        
        // Compress and save
        let compressor = ImageCompressor::new(self.config.quality);
        compressor.save(&image, output_path.as_ref())?;
        
        Ok(())
    }
    
    pub fn process_single<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        self.process(input_path, output_path)
    }
}