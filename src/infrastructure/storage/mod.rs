//! Storage infrastructure module
//!
//! Provides file storage operations with:
//! - Local filesystem storage
//! - Metadata stripping for privacy
//! - Image processing and optimization
//! - Content hashing for deduplication

use image::{DynamicImage, GenericImageView, ImageFormat, ImageOutputFormat};
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::StorageSettings;
use crate::errors::ApiError;

/// Storage service for file operations
#[derive(Clone)]
pub struct StorageService {
    /// Base storage path
    storage_path: PathBuf,
    /// Maximum allowed file size
    max_file_size: usize,
    /// Allowed MIME types
    allowed_mime_types: Vec<String>,
}

impl StorageService {
    /// Create a new storage service
    pub async fn new(settings: &StorageSettings) -> Result<Self, ApiError> {
        let storage_path = PathBuf::from(&settings.local_path);

        // Create storage directory if it doesn't exist
        fs::create_dir_all(&storage_path).await.map_err(|e| {
            error!(error = %e, path = %storage_path.display(), "Failed to create storage directory");
            ApiError::StorageError("Failed to create storage directory".to_string())
        })?;

        // Create subdirectories for organization
        let subdirs = ["images", "thumbnails", "temp"];
        for subdir in subdirs {
            let subpath = storage_path.join(subdir);
            fs::create_dir_all(&subpath).await.map_err(|e| {
                error!(error = %e, path = %subpath.display(), "Failed to create subdirectory");
                ApiError::StorageError("Failed to create storage subdirectory".to_string())
            })?;
        }

        info!(path = %storage_path.display(), "Storage service initialized");

        Ok(Self {
            storage_path,
            max_file_size: settings.max_file_size,
            allowed_mime_types: settings.allowed_mime_types.clone(),
        })
    }

    /// Store an image file with metadata stripping
    pub async fn store_image(
        &self,
        data: &[u8],
        original_filename: Option<&str>,
        mime_type: &str,
    ) -> Result<StoredFile, ApiError> {
        // Validate file size
        if data.len() > self.max_file_size {
            return Err(ApiError::FileTooLarge(self.max_file_size));
        }

        // Validate MIME type
        if !self.allowed_mime_types.contains(&mime_type.to_string()) {
            return Err(ApiError::InvalidFileType(format!(
                "MIME type '{}' is not allowed",
                mime_type
            )));
        }

        // Parse and process the image to strip metadata
        let (processed_data, format) = self.process_image(data, mime_type)?;

        // Calculate content hash
        let content_hash = Self::calculate_hash(&processed_data);

        // Generate unique filename
        let file_id = Uuid::new_v4();
        let extension = Self::extension_from_format(&format);
        let filename = format!("{}.{}", file_id, extension);

        // Store the file
        let relative_path = format!("images/{}", filename);
        let full_path = self.storage_path.join(&relative_path);

        let mut file = fs::File::create(&full_path).await.map_err(|e| {
            error!(error = %e, path = %full_path.display(), "Failed to create file");
            ApiError::StorageError("Failed to store file".to_string())
        })?;

        file.write_all(&processed_data).await.map_err(|e| {
            error!(error = %e, "Failed to write file data");
            ApiError::StorageError("Failed to write file data".to_string())
        })?;

        file.sync_all().await.map_err(|e| {
            error!(error = %e, "Failed to sync file");
            ApiError::StorageError("Failed to sync file".to_string())
        })?;

        // Generate thumbnail
        let thumbnail_path = self.generate_thumbnail(&processed_data, &file_id, &format).await?;

        debug!(
            file_id = %file_id,
            size = processed_data.len(),
            hash = %content_hash,
            "Image stored successfully"
        );

        Ok(StoredFile {
            id: file_id,
            path: relative_path,
            thumbnail_path,
            content_hash,
            mime_type: Self::mime_from_format(&format),
            size: processed_data.len(),
        })
    }

    /// Process an image: decode, strip metadata, re-encode
    fn process_image(&self, data: &[u8], mime_type: &str) -> Result<(Vec<u8>, ImageFormat), ApiError> {
        // Determine the format from MIME type
        let format = Self::format_from_mime(mime_type)?;

        // Load the image (this strips EXIF and other metadata)
        let img = image::load_from_memory_with_format(data, format).map_err(|e| {
            error!(error = %e, "Failed to load image");
            ApiError::FileProcessingFailed("Invalid image data".to_string())
        })?;

        // Re-encode the image without metadata
        let mut output = Vec::new();
        let cursor = Cursor::new(&mut output);

        // Choose output format and quality
        let output_format = match format {
            ImageFormat::Png => ImageFormat::Png,
            ImageFormat::Gif => ImageFormat::Gif,
            ImageFormat::WebP => ImageFormat::WebP,
            _ => ImageFormat::Jpeg, // Default to JPEG for other formats
        };

        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(cursor, 85);

        match output_format {
            ImageFormat::Png => {
                let mut cursor = Cursor::new(&mut output);
                img.write_to(&mut cursor, ImageOutputFormat::Png)
                    .map_err(|e| {
                        error!(error = %e, "Failed to encode PNG");
                        ApiError::FileProcessingFailed("Failed to process image".to_string())
                    })?;
            }
            ImageFormat::Gif => {
                let mut cursor = Cursor::new(&mut output);
                img.write_to(&mut cursor, ImageOutputFormat::Gif)
                    .map_err(|e| {
                        error!(error = %e, "Failed to encode GIF");
                        ApiError::FileProcessingFailed("Failed to process image".to_string())
                    })?;
            }
            ImageFormat::WebP => {
                // WebP encoding - fall back to JPEG if not supported
                let mut cursor = Cursor::new(&mut output);
                img.write_to(&mut cursor, ImageOutputFormat::Jpeg(85))
                    .map_err(|e| {
                        error!(error = %e, "Failed to encode image");
                        ApiError::FileProcessingFailed("Failed to process image".to_string())
                    })?;
            }
            _ => {
                let mut cursor = Cursor::new(&mut output);
                img.write_to(&mut cursor, ImageOutputFormat::Jpeg(85))
                    .map_err(|e| {
                        error!(error = %e, "Failed to encode JPEG");
                        ApiError::FileProcessingFailed("Failed to process image".to_string())
                    })?;
            }
        }

        Ok((output, output_format))
    }

    /// Generate a thumbnail for an image
    async fn generate_thumbnail(
        &self,
        data: &[u8],
        file_id: &Uuid,
        format: &ImageFormat,
    ) -> Result<Option<String>, ApiError> {
        const THUMBNAIL_SIZE: u32 = 200;

        let img = image::load_from_memory(data).map_err(|e| {
            warn!(error = %e, "Failed to load image for thumbnail");
            ApiError::FileProcessingFailed("Failed to generate thumbnail".to_string())
        })?;

        // Only generate thumbnails for images larger than the thumbnail size
        let (width, height) = img.dimensions();
        if width <= THUMBNAIL_SIZE && height <= THUMBNAIL_SIZE {
            return Ok(None);
        }

        // Create thumbnail
        let thumbnail = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);

        // Encode thumbnail
        let mut output = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        thumbnail
            .write_to(&mut cursor, ImageOutputFormat::Jpeg(75))
            .map_err(|e| {
                warn!(error = %e, "Failed to encode thumbnail");
                ApiError::FileProcessingFailed("Failed to generate thumbnail".to_string())
            })?;

        // Save thumbnail
        let extension = Self::extension_from_format(format);
        let thumbnail_filename = format!("{}_thumb.{}", file_id, extension);
        let relative_path = format!("thumbnails/{}", thumbnail_filename);
        let full_path = self.storage_path.join(&relative_path);

        let mut file = fs::File::create(&full_path).await.map_err(|e| {
            error!(error = %e, "Failed to create thumbnail file");
            ApiError::StorageError("Failed to store thumbnail".to_string())
        })?;

        file.write_all(&output).await.map_err(|e| {
            error!(error = %e, "Failed to write thumbnail data");
            ApiError::StorageError("Failed to write thumbnail".to_string())
        })?;

        Ok(Some(relative_path))
    }

    /// Resolve a relative path and ensure it stays within the storage directory.
    /// Uses canonicalization to prevent path traversal attacks like `../../etc/passwd`.
    fn resolve_safe_path(&self, relative_path: &str) -> Result<PathBuf, ApiError> {
        // Reject obvious traversal patterns early
        if relative_path.contains("..") || relative_path.starts_with('/') || relative_path.starts_with('\\') {
            return Err(ApiError::InvalidInput("Invalid file path".to_string()));
        }

        let full_path = self.storage_path.join(relative_path);

        // Canonicalize both paths to resolve symlinks and normalize components.
        // For get/delete, the file must already exist for canonicalize to work,
        // so we fall back to checking the raw join result.
        let canonical_storage = self.storage_path.canonicalize().unwrap_or_else(|_| self.storage_path.clone());
        let canonical_full = full_path.canonicalize().unwrap_or(full_path.clone());

        if !canonical_full.starts_with(&canonical_storage) {
            warn!(
                attempted_path = %relative_path,
                resolved = %canonical_full.display(),
                "Path traversal attempt blocked"
            );
            return Err(ApiError::InvalidInput("Invalid file path".to_string()));
        }

        Ok(canonical_full)
    }

    /// Get a file by its relative path
    pub async fn get_file(&self, relative_path: &str) -> Result<Vec<u8>, ApiError> {
        let full_path = self.resolve_safe_path(relative_path)?;

        fs::read(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ApiError::NotFound("File not found".to_string())
            } else {
                error!(error = %e, path = %full_path.display(), "Failed to read file");
                ApiError::StorageError("Failed to read file".to_string())
            }
        })
    }

    /// Delete a file
    pub async fn delete_file(&self, relative_path: &str) -> Result<(), ApiError> {
        let full_path = self.resolve_safe_path(relative_path)?;

        match fs::remove_file(&full_path).await {
            Ok(_) => {
                debug!(path = %relative_path, "File deleted");
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File already doesn't exist — consider this success
                Ok(())
            }
            Err(e) => {
                error!(error = %e, path = %full_path.display(), "Failed to delete file");
                Err(ApiError::StorageError("Failed to delete file".to_string()))
            }
        }
    }

    /// Check if a file exists
    pub async fn file_exists(&self, relative_path: &str) -> bool {
        match self.resolve_safe_path(relative_path) {
            Ok(full_path) => full_path.exists() && full_path.is_file(),
            Err(_) => false,
        }
    }

    /// Get file metadata
    pub async fn get_metadata(&self, relative_path: &str) -> Result<FileMetadata, ApiError> {
        let full_path = self.resolve_safe_path(relative_path)?;

        let metadata = fs::metadata(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ApiError::NotFound("File not found".to_string())
            } else {
                error!(error = %e, "Failed to get file metadata");
                ApiError::StorageError("Failed to get file metadata".to_string())
            }
        })?;

        let modified = metadata.modified().ok().map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
        });

        Ok(FileMetadata {
            size: metadata.len() as usize,
            modified,
            is_file: metadata.is_file(),
        })
    }

    /// Calculate SHA-256 hash of data
    fn calculate_hash(data: &[u8]) -> String {
        hex::encode(Sha256::digest(data))
    }

    /// Get ImageFormat from MIME type
    fn format_from_mime(mime_type: &str) -> Result<ImageFormat, ApiError> {
        match mime_type {
            "image/jpeg" | "image/jpg" => Ok(ImageFormat::Jpeg),
            "image/png" => Ok(ImageFormat::Png),
            "image/gif" => Ok(ImageFormat::Gif),
            "image/webp" => Ok(ImageFormat::WebP),
            _ => Err(ApiError::InvalidFileType(format!(
                "Unsupported image type: {}",
                mime_type
            ))),
        }
    }

    /// Get MIME type from ImageFormat
    fn mime_from_format(format: &ImageFormat) -> String {
        match format {
            ImageFormat::Jpeg => "image/jpeg".to_string(),
            ImageFormat::Png => "image/png".to_string(),
            ImageFormat::Gif => "image/gif".to_string(),
            ImageFormat::WebP => "image/webp".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    /// Get file extension from ImageFormat
    fn extension_from_format(format: &ImageFormat) -> &'static str {
        match format {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
            ImageFormat::Gif => "gif",
            ImageFormat::WebP => "webp",
            _ => "bin",
        }
    }

    /// Clean up temporary files older than specified duration
    pub async fn cleanup_temp_files(&self, max_age: std::time::Duration) -> Result<usize, ApiError> {
        let temp_dir = self.storage_path.join("temp");
        let mut deleted_count = 0;

        let mut entries = fs::read_dir(&temp_dir).await.map_err(|e| {
            error!(error = %e, "Failed to read temp directory");
            ApiError::StorageError("Failed to read temp directory".to_string())
        })?;

        let now = std::time::SystemTime::now();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            error!(error = %e, "Failed to read directory entry");
            ApiError::StorageError("Failed to read directory".to_string())
        })? {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > max_age {
                                if fs::remove_file(&path).await.is_ok() {
                                    deleted_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        info!(deleted = deleted_count, "Cleaned up temporary files");
        Ok(deleted_count)
    }
}

/// Information about a stored file
#[derive(Debug, Clone)]
pub struct StoredFile {
    /// Unique file ID
    pub id: Uuid,
    /// Relative path to the file
    pub path: String,
    /// Relative path to thumbnail (if generated)
    pub thumbnail_path: Option<String>,
    /// SHA-256 hash of the content
    pub content_hash: String,
    /// MIME type
    pub mime_type: String,
    /// File size in bytes
    pub size: usize,
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: usize,
    /// Last modified time
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether this is a file (vs directory)
    pub is_file: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would require a temporary directory
}
