use crate::{ImageToolError, ProcessConfig, Result};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

pub struct BatchProcessor {
    config: ProcessConfig,
    max_threads: usize,
}

impl BatchProcessor {
    pub fn new(config: ProcessConfig, max_threads: usize) -> Self {
        Self { config, max_threads }
    }
    
    pub fn process_directory(
        &self,
        input_dir: &Path,
        output_dir: &Path,
        recursive: bool,
    ) -> Result<usize> {
        // Set up rayon thread pool if custom thread count is specified
        if self.max_threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(self.max_threads)
                .build_global()
                .map_err(|e| {
                    ImageToolError::ProcessingError(format!("Failed to create thread pool: {}", e))
                })?;
        }
        
        // Collect image files
        let image_paths = self.collect_image_paths(input_dir, recursive)?;
        
        if image_paths.is_empty() {
            log::warn!("No image files found in {}", input_dir.display());
            return Ok(0);
        }
        
        log::info!(
            "Processing {} images from {}",
            image_paths.len(),
            input_dir.display()
        );
        
        // Create progress bar
        let pb = ProgressBar::new(image_paths.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        
        // Create output directory
        std::fs::create_dir_all(output_dir)?;
        
        // Process images in parallel
        let config = Arc::new(self.config.clone());
        let processed_count: usize = image_paths
            .par_iter()
            .progress_with(pb.clone())
            .map(|input_path| {
                self.process_single_image(input_path, output_dir, config.as_ref())
                    .unwrap_or_else(|e| {
                        log::warn!("Failed to process {}: {}", input_path.display(), e);
                        0
                    })
            })
            .sum();
        
        pb.finish_with_message(format!("Processed {} images", processed_count));
        
        Ok(processed_count)
    }
    
    fn process_single_image(
        &self,
        input_path: &Path,
        output_dir: &Path,
        config: &ProcessConfig,
    ) -> Result<usize> {
        use crate::ImageProcessor;
        
        // Calculate output path
        let file_name = input_path
            .file_name()
            .ok_or_else(|| {
                ImageToolError::InvalidParameter(format!("Invalid file name: {}", input_path.display()))
            })?;
        
        let output_path = output_dir.join(file_name);
        
        // Create processor and process
        let processor = ImageProcessor::new(config.clone());
        processor.process(input_path, &output_path)?;
        
        Ok(1)
    }
    
    fn collect_image_paths(&self, input_dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
        let walker = if recursive {
            WalkDir::new(input_dir)
        } else {
            WalkDir::new(input_dir).max_depth(1)
        };
        
        let image_extensions = [
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp",
        ];
        
        let paths: Vec<PathBuf> = walker
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| image_extensions.contains(&ext.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .map(|entry| entry.into_path())
            .collect();
        
        Ok(paths)
    }
    
    pub fn validate_paths(&self, input_dir: &Path, output_dir: &Path) -> Result<()> {
        if !input_dir.exists() {
            return Err(ImageToolError::InvalidParameter(
                format!("Input directory does not exist: {}", input_dir.display())
            ));
        }
        
        if !input_dir.is_dir() {
            return Err(ImageToolError::InvalidParameter(
                format!("Input path is not a directory: {}", input_dir.display())
            ));
        }
        
        if output_dir.exists() && !output_dir.is_dir() {
            return Err(ImageToolError::InvalidParameter(
                format!("Output path exists but is not a directory: {}", output_dir.display())
            ));
        }
        
        Ok(())
    }
}