// pixie/src/cli.rs
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum Algorithm {
    Nearest,
    Bilinear,
    Bicubic,
    Lanczos3,
}

impl From<Algorithm> for crate::ResizeAlgorithm {
    fn from(value: Algorithm) -> Self {
        match value {
            Algorithm::Nearest => crate::ResizeAlgorithm::Nearest,
            Algorithm::Bilinear => crate::ResizeAlgorithm::Bilinear,
            Algorithm::Bicubic => crate::ResizeAlgorithm::Bicubic,
            Algorithm::Lanczos3 => crate::ResizeAlgorithm::Lanczos3,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Jpeg,
    Png,
    WebP,
    Same,
}

impl From<OutputFormat> for crate::OutputFormat {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Jpeg => crate::OutputFormat::Jpeg,
            OutputFormat::Png => crate::OutputFormat::Png,
            OutputFormat::WebP => crate::OutputFormat::WebP,
            OutputFormat::Same => crate::OutputFormat::SameAsInput,
        }
    }
}

#[derive(Parser)]
#[command(name = "image-tool")]
#[command(version, about = "Fast image resizer and optimizer", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Maximum file size to process (in MB)
    #[arg(long, global = true, value_name = "MB")]
    pub max_file_size: Option<u64>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Resize a single image
    Resize {
        /// Input image file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output image file (default: input_resized.ext)
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Width in pixels (0 for auto)
        #[arg(short = 'W', long, default_value_t = 0, value_name = "PIXELS")]
        width: u32,

        /// Height in pixels (0 for auto)
        #[arg(short = 'H', long, default_value_t = 0, value_name = "PIXELS")]
        height: u32,

        /// Scale percentage (overrides width/height)
        #[arg(short, long, default_value_t = 0.0, value_name = "PERCENT")]
        scale: f32,

        /// JPEG quality (1-100)
        #[arg(short, long, default_value_t = 85, value_name = "QUALITY")]
        quality: u8,

        /// Output format
        #[arg(short = 'f', long, value_enum, value_name = "FORMAT")]
        format: Option<OutputFormat>,

        /// Maintain aspect ratio
        #[arg(short = 'a', long)]
        keep_aspect: bool,

        /// Strip metadata (EXIF, etc.)
        #[arg(short = 'm', long)]
        strip_metadata: bool,

        /// Resize algorithm
        #[arg(short = 'A', long, value_enum, default_value_t = Algorithm::Lanczos3)]
        algorithm: Algorithm,

        /// Use progressive JPEG encoding
        #[arg(long)]
        progressive: bool,
    },

    /// Process multiple images in a folder
    Batch {
        /// Input directory
        #[arg(value_name = "INPUT_DIR")]
        input: PathBuf,

        /// Output directory
        #[arg(short, long, value_name = "OUTPUT_DIR")]
        output: PathBuf,

        /// Width in pixels
        #[arg(short = 'W', long, default_value_t = 800, value_name = "PIXELS")]
        width: u32,

        /// Height in pixels (0 for auto)
        #[arg(short = 'H', long, default_value_t = 0, value_name = "PIXELS")]
        height: u32,

        /// Output format
        #[arg(short = 'f', long, value_enum, value_name = "FORMAT")]
        format: Option<OutputFormat>,

        /// JPEG quality (1-100)
        #[arg(short, long, default_value_t = 85, value_name = "QUALITY")]
        quality: u8,

        /// Number of parallel threads (0 = auto)
        #[arg(short, long, default_value_t = 0, value_name = "THREADS")]
        threads: usize,

        /// Recursively process subdirectories
        #[arg(short, long)]
        recursive: bool,

        /// Strip metadata
        #[arg(short = 'm', long)]
        strip_metadata: bool,

        /// Resize algorithm
        #[arg(short = 'A', long, value_enum, default_value_t = Algorithm::Lanczos3)]
        algorithm: Algorithm,

        /// Disable PNG optimization
        #[arg(long)]
        no_png_optimize: bool,
    },

    /// Optimize image without resizing
    Optimize {
        /// Input image file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output image file
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// JPEG quality (1-100)
        #[arg(short, long, default_value_t = 85, value_name = "QUALITY")]
        quality: u8,

        /// Strip metadata
        #[arg(short = 'm', long)]
        strip_metadata: bool,

        /// Use progressive JPEG encoding
        #[arg(long)]
        progressive: bool,

        /// Disable PNG optimization
        #[arg(long)]
        no_png_optimize: bool,
    },

    /// Get information about an image
    Info {
        /// Input image file
        input: PathBuf,

        /// Show detailed EXIF metadata
        #[arg(short, long)]
        exif: bool,
    },

    /// Convert image to another format
    Convert {
        /// Input image file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output image file (default: input_converted.ext)
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Target format
        #[arg(short = 'f', long, value_enum, value_name = "FORMAT")]
        format: OutputFormat,

        /// JPEG quality (1-100) for JPEG output
        #[arg(short, long, default_value_t = 85, value_name = "QUALITY")]
        quality: u8,

        /// Strip metadata
        #[arg(short = 'm', long)]
        strip_metadata: bool,
    },
}