// pixie/src/main.rs
use image_tool::prelude::*;
use image_tool::{Cli, Commands, Algorithm, OutputFormat};
use clap::Parser;
use log::LevelFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logger
    env_logger::Builder::new()
        .filter_level(if cli.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init();

    let max_file_size = cli.max_file_size.map(|mb| mb * 1024 * 1024);

    match cli.command {
        Commands::Resize {
            input,
            output,
            width,
            height,
            scale,
            quality,
            format,
            keep_aspect,
            strip_metadata,
            algorithm,
            progressive,
        } => {
            process_resize(
                input, output, width, height, scale, quality,
                format, keep_aspect, strip_metadata, algorithm,
                progressive, max_file_size,
            )?;
        }
        Commands::Batch {
            input,
            output,
            width,
            height,
            format,
            quality,
            threads,
            recursive,
            strip_metadata,
            algorithm,
            no_png_optimize,
        } => {
            process_batch(
                input, output, width, height, format, quality,
                threads, recursive, strip_metadata, algorithm,
                no_png_optimize, max_file_size,
            )?;
        }
        Commands::Optimize {
            input,
            output,
            quality,
            strip_metadata,
            progressive,
            no_png_optimize,
        } => {
            process_optimize(
                input, output, quality, strip_metadata,
                progressive, no_png_optimize, max_file_size,
            )?;
        }
        Commands::Info { input, exif } => {
            process_info(input, exif)?;
        }
        Commands::Convert {
            input,
            output,
            format,
            quality,
            strip_metadata,
        } => {
            process_convert(
                input, output, format, quality,
                strip_metadata, max_file_size,
            )?;
        }
    }

    Ok(())
}

fn process_resize(
    input: std::path::PathBuf,
    output: Option<std::path::PathBuf>,
    width: u32,
    height: u32,
    scale: f32,
    quality: u8,
    format: Option<OutputFormat>,
    keep_aspect: bool,
    strip_metadata: bool,
    algorithm: Algorithm,
    progressive: bool,
    max_file_size: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::utils::generate_output_path;
    
    let output_path = generate_output_path(&input, output.as_deref(), "resized");

    let config = ProcessConfig {
        width,
        height,
        scale,
        quality,
        keep_aspect,
        strip_metadata,
        algorithm: algorithm.into(),
        max_file_size,
        format: format.map(|f| f.into()),
        ..Default::default()
    };

    config.validate()?;

    let processor = ImageProcessor::new(config);
    let stats = processor.process(&input, &output_path)?;

    println!("✓ Resized image saved to: {}", output_path.display());
    print_stats(&stats);

    Ok(())
}

fn process_batch(
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    width: u32,
    height: u32,
    format: Option<OutputFormat>,
    quality: u8,
    threads: usize,
    recursive: bool,
    strip_metadata: bool,
    algorithm: Algorithm,
    no_png_optimize: bool,
    max_file_size: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ProcessConfig {
        width,
        height,
        scale: 0.0,
        quality,
        keep_aspect: true,
        strip_metadata,
        algorithm: algorithm.into(),
        max_file_size,
        format: format.map(|f| f.into()),
    };

    config.validate()?;

    let processor = BatchProcessor::new(config, threads)?;
    processor.validate_paths(&input, &output)?;

    let stats = processor.process_directory(&input, &output, recursive)?;

    println!("✓ Batch processing complete.");
    print_stats(&stats);

    if !stats.errors.is_empty() {
        println!("\n⚠  Errors encountered:");
        for (context, error) in &stats.errors {
            println!("  - {}: {}", context, error);
        }
    }

    Ok(())
}

fn process_optimize(
    input: std::path::PathBuf,
    output: Option<std::path::PathBuf>,
    quality: u8,
    strip_metadata: bool,
    progressive: bool,
    no_png_optimize: bool,
    max_file_size: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::utils::generate_output_path;
    
    let output_path = generate_output_path(&input, output.as_deref(), "optimized");

    let config = ProcessConfig {
        width: 0,
        height: 0,
        scale: 0.0,
        quality,
        keep_aspect: true,
        strip_metadata,
        algorithm: ResizeAlgorithm::Lanczos3,
        max_file_size,
        format: None,
    };

    config.validate()?;

    let processor = ImageProcessor::new(config);
    let stats = processor.process(&input, &output_path)?;

    println!("✓ Optimized image saved to: {}", output_path.display());
    print_stats(&stats);

    Ok(())
}

fn process_info(
    input: std::path::PathBuf,
    exif: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::utils::{format_file_size, get_image_info};
    
    if !input.exists() {
        return Err(format!("File does not exist: {}", input.display()).into());
    }

    let processor = ImageProcessor::new(ProcessConfig::default());
    let metadata = processor.get_metadata(&input)?;

    println!("=== Image Information ===");
    println!("File: {}", input.display());
    println!("Size: {}", format_file_size(metadata.file_size));
    println!("Dimensions: {} × {} pixels", metadata.width, metadata.height);
    println!("Aspect Ratio: {:.2}:1", metadata.width as f32 / metadata.height as f32);
    println!("Format: {}", metadata.format);
    println!("Has EXIF metadata: {}", metadata.has_exif);

    if exif && metadata.has_exif {
        let metadata_processor = MetadataProcessor::new();
        if let Ok(Some(exif_data)) = metadata_processor.read_metadata(&input) {
            println!("\n{}", metadata_processor.print_metadata(&exif_data));
        }
    }

    Ok(())
}

fn process_convert(
    input: std::path::PathBuf,
    output: Option<std::path::PathBuf>,
    format: OutputFormat,
    quality: u8,
    strip_metadata: bool,
    max_file_size: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::utils::generate_output_path;
    
    let output_path = generate_output_path(&input, output.as_deref(), "converted");

    let config = ProcessConfig {
        width: 0,
        height: 0,
        scale: 0.0,
        quality,
        keep_aspect: true,
        strip_metadata,
        algorithm: ResizeAlgorithm::Lanczos3,
        max_file_size,
        format: Some(format.into()),
    };

    config.validate()?;

    let processor = ImageProcessor::new(config);
    let stats = processor.process(&input, &output_path)?;

    println!("✓ Converted image saved to: {}", output_path.display());
    print_stats(&stats);

    Ok(())
}

fn print_stats(stats: &ProcessingStats) {
    if stats.processed_count > 0 && stats.total_size_before > 0 {
        let reduction = if stats.total_size_after < stats.total_size_before {
            let percent = (stats.total_size_before - stats.total_size_after) as f64 
                / stats.total_size_before as f64 * 100.0;
            format!(" (reduced by {:.1}%)", percent)
        } else {
            String::new()
        };
        
        println!("  Processed: {} file(s)", stats.processed_count);
        println!("  Original size: {}", format_file_size(stats.total_size_before));
        println!("  Final size: {}{}", format_file_size(stats.total_size_after), reduction);
    }
}