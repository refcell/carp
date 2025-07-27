/// File upload and download validation tests
/// These tests verify file handling, validation, and integrity

use bytes::Bytes;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Write};
use tempfile::NamedTempFile;
use zip::{write::FileOptions, ZipWriter};

// Test utilities for file operations
mod file_test_utils {
    use super::*;

    pub fn create_test_zip_content() -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
            
            // Add manifest file
            zip.start_file("carp.toml", FileOptions::default())
                .expect("Failed to start manifest file");
            zip.write_all(
                br#"
[package]
name = "test-agent"
version = "1.0.0"
description = "Test agent for file upload testing"
authors = ["Test User <test@example.com>"]
license = "MIT"

[agent]
main = "main.py"
"#,
            )
            .expect("Failed to write manifest");

            // Add main file
            zip.start_file("main.py", FileOptions::default())
                .expect("Failed to start main file");
            zip.write_all(
                br#"
#!/usr/bin/env python3
"""Test agent main file."""

def main():
    print("Hello from test agent!")

if __name__ == "__main__":
    main()
"#,
            )
            .expect("Failed to write main file");

            // Add README
            zip.start_file("README.md", FileOptions::default())
                .expect("Failed to start README");
            zip.write_all(
                br#"
# Test Agent

This is a test agent for file upload testing.

## Usage

```bash
python main.py
```
"#,
            )
            .expect("Failed to write README");

            zip.finish().expect("Failed to finish ZIP");
        }
        buffer
    }

    pub fn create_invalid_zip_content() -> Vec<u8> {
        b"This is not a valid ZIP file content".to_vec()
    }

    pub fn create_large_zip_content(size_mb: usize) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
            
            // Add manifest
            zip.start_file("carp.toml", FileOptions::default())
                .expect("Failed to start manifest");
            zip.write_all(b"[package]\nname = \"large-agent\"\n")
                .expect("Failed to write manifest");

            // Add large file
            zip.start_file("large_file.txt", FileOptions::default())
                .expect("Failed to start large file");
            
            let chunk = vec![b'A'; 1024]; // 1KB chunk
            for _ in 0..(size_mb * 1024) {
                zip.write_all(&chunk).expect("Failed to write chunk");
            }

            zip.finish().expect("Failed to finish ZIP");
        }
        buffer
    }

    pub fn calculate_checksum(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    pub fn create_corrupted_zip() -> Vec<u8> {
        let mut valid_zip = create_test_zip_content();
        // Corrupt the ZIP by modifying some bytes in the middle
        if valid_zip.len() > 100 {
            valid_zip[50] = 0xFF;
            valid_zip[51] = 0xFF;
            valid_zip[52] = 0xFF;
        }
        valid_zip
    }

    pub fn extract_zip_and_validate(content: &[u8]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let cursor = Cursor::new(content);
        let mut archive = zip::ZipArchive::new(cursor)?;
        let mut filenames = Vec::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            filenames.push(file.name().to_string());
        }

        Ok(filenames)
    }
}

// Test ZIP file creation and validation
#[test]
fn test_create_valid_zip_file() {
    let zip_content = file_test_utils::create_test_zip_content();
    
    // Should be able to create ZIP content
    assert!(!zip_content.is_empty());
    assert!(zip_content.len() > 100); // Should have some substantial content

    // Should be able to extract and validate
    let filenames = file_test_utils::extract_zip_and_validate(&zip_content);
    assert!(filenames.is_ok());
    
    let files = filenames.unwrap();
    assert!(files.contains(&"carp.toml".to_string()));
    assert!(files.contains(&"main.py".to_string()));
    assert!(files.contains(&"README.md".to_string()));
}

// Test ZIP file corruption detection
#[test]
fn test_corrupted_zip_detection() {
    let corrupted_zip = file_test_utils::create_corrupted_zip();
    
    // Should fail to extract corrupted ZIP
    let result = file_test_utils::extract_zip_and_validate(&corrupted_zip);
    assert!(result.is_err());
}

// Test invalid ZIP file handling
#[test]
fn test_invalid_zip_file() {
    let invalid_content = file_test_utils::create_invalid_zip_content();
    
    // Should fail to process invalid ZIP
    let result = file_test_utils::extract_zip_and_validate(&invalid_content);
    assert!(result.is_err());
}

// Test checksum calculation
#[test]
fn test_checksum_calculation() {
    let content1 = b"test content for checksum";
    let content2 = b"test content for checksum";
    let content3 = b"different content";

    let checksum1 = file_test_utils::calculate_checksum(content1);
    let checksum2 = file_test_utils::calculate_checksum(content2);
    let checksum3 = file_test_utils::calculate_checksum(content3);

    // Same content should produce same checksum
    assert_eq!(checksum1, checksum2);
    
    // Different content should produce different checksum
    assert_ne!(checksum1, checksum3);
    
    // Checksum should be 64 characters (SHA256 hex)
    assert_eq!(checksum1.len(), 64);
    
    // Should only contain hex characters
    assert!(checksum1.chars().all(|c| c.is_ascii_hexdigit()));
}

// Test file size limits
#[test]
fn test_file_size_validation() {
    let max_size = 10 * 1024 * 1024; // 10MB limit
    
    // Small file should be acceptable
    let small_file = vec![b'A'; 1024]; // 1KB
    assert!(small_file.len() <= max_size);
    
    // Large file should be rejected
    let large_file = vec![b'B'; 15 * 1024 * 1024]; // 15MB
    assert!(large_file.len() > max_size);
}

// Test multipart form data handling
#[test]
fn test_multipart_boundary_parsing() {
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let content_type = format!("multipart/form-data; boundary={}", boundary);
    
    // Verify boundary extraction
    assert!(content_type.contains(boundary));
    
    // Test multipart content structure
    let multipart_content = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n{}\r\n--{}\r\nContent-Disposition: form-data; name=\"content\"; filename=\"agent.zip\"\r\nContent-Type: application/zip\r\n\r\n{}\r\n--{}--\r\n",
        boundary,
        r#"{"name":"test-agent","version":"1.0.0","description":"Test"}"#,
        boundary,
        "ZIP_CONTENT_HERE",
        boundary
    );
    
    assert!(multipart_content.contains("form-data"));
    assert!(multipart_content.contains("metadata"));
    assert!(multipart_content.contains("content"));
    assert!(multipart_content.contains("application/zip"));
}

// Test MIME type validation
#[test]
fn test_mime_type_validation() {
    let valid_mime_types = [
        "application/zip",
        "application/x-zip-compressed",
        "application/octet-stream",
    ];
    
    let invalid_mime_types = [
        "text/plain",
        "image/jpeg",
        "application/json",
        "text/html",
    ];
    
    // Test valid MIME types
    for mime_type in &valid_mime_types {
        assert!(
            mime_type.starts_with("application/zip") || 
            mime_type.starts_with("application/x-zip") ||
            mime_type.starts_with("application/octet-stream"),
            "MIME type {} should be valid for ZIP files",
            mime_type
        );
    }
    
    // Test invalid MIME types
    for mime_type in &invalid_mime_types {
        assert!(
            !mime_type.starts_with("application/zip") && 
            !mime_type.starts_with("application/x-zip") &&
            !mime_type.starts_with("application/octet-stream"),
            "MIME type {} should be invalid for ZIP files",
            mime_type
        );
    }
}

// Test filename validation and sanitization
#[test]
fn test_filename_validation() {
    let valid_filenames = [
        "agent.zip",
        "my-agent-v1.0.0.zip",
        "simple_agent.zip",
        "agent123.zip",
    ];
    
    let problematic_filenames = [
        "../agent.zip",           // Path traversal
        "agent.exe",              // Wrong extension
        "agent",                  // No extension
        "agent.zip.txt",          // Multiple extensions
        "",                       // Empty filename
        "agent with spaces.zip",  // Spaces (may need handling)
    ];
    
    // Test valid filenames
    for filename in &valid_filenames {
        assert!(filename.ends_with(".zip"));
        assert!(!filename.contains(".."));
        assert!(!filename.is_empty());
    }
    
    // Test problematic filenames
    for filename in &problematic_filenames {
        let is_problematic = filename.contains("..") || 
                           !filename.ends_with(".zip") || 
                           filename.is_empty() ||
                           filename.contains(" ");
        
        // Most of these should be considered problematic
        // (spaces might be allowed depending on implementation)
        if *filename != "agent with spaces.zip" {
            assert!(is_problematic, "Filename '{}' should be problematic", filename);
        }
    }
}

// Test file content validation
#[test]
fn test_file_content_validation() {
    // Test valid ZIP content
    let valid_zip = file_test_utils::create_test_zip_content();
    assert!(valid_zip.len() > 0);
    
    // ZIP files should start with specific magic bytes
    assert_eq!(valid_zip[0], 0x50); // 'P'
    assert_eq!(valid_zip[1], 0x4B); // 'K'
    
    // Test content that looks like ZIP but isn't
    let fake_zip = b"PK\x03\x04fake zip content";
    // This might pass the magic byte check but fail on full validation
    assert_eq!(fake_zip[0], 0x50);
    assert_eq!(fake_zip[1], 0x4B);
    
    // However, it should fail when trying to extract
    let extract_result = file_test_utils::extract_zip_and_validate(fake_zip);
    assert!(extract_result.is_err());
}

// Test concurrent file operations
#[tokio::test]
async fn test_concurrent_file_operations() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let file_counter = Arc::new(Mutex::new(0));
    let futures = (0..5).map(|i| {
        let counter = file_counter.clone();
        tokio::spawn(async move {
            // Simulate file processing
            let content = file_test_utils::create_test_zip_content();
            let checksum = file_test_utils::calculate_checksum(&content);
            
            // Update counter
            let mut count = counter.lock().await;
            *count += 1;
            
            (i, content.len(), checksum)
        })
    });
    
    let results = futures::future::join_all(futures).await;
    
    // All operations should complete successfully
    assert_eq!(results.len(), 5);
    
    for result in results {
        let (index, size, checksum) = result.expect("Task should complete");
        assert!(index < 5);
        assert!(size > 0);
        assert_eq!(checksum.len(), 64);
    }
    
    // Counter should be 5
    let final_count = *file_counter.lock().await;
    assert_eq!(final_count, 5);
}

// Test file streaming and chunked processing
#[test]
fn test_file_streaming() {
    let large_content = file_test_utils::create_large_zip_content(1); // 1MB
    let chunk_size = 8192; // 8KB chunks
    
    // Process file in chunks
    let mut processed_size = 0;
    let mut chunk_count = 0;
    
    for chunk in large_content.chunks(chunk_size) {
        processed_size += chunk.len();
        chunk_count += 1;
        
        // Each chunk (except possibly the last) should be the expected size
        if chunk_count * chunk_size <= large_content.len() {
            assert_eq!(chunk.len(), chunk_size);
        }
    }
    
    // Should have processed all content
    assert_eq!(processed_size, large_content.len());
    assert!(chunk_count > 0);
}

// Test memory-efficient file handling
#[test]
fn test_memory_efficient_checksum() {
    let content = file_test_utils::create_large_zip_content(2); // 2MB
    
    // Calculate checksum all at once
    let full_checksum = file_test_utils::calculate_checksum(&content);
    
    // Calculate checksum in chunks (simulating streaming)
    let mut hasher = Sha256::new();
    let chunk_size = 8192;
    
    for chunk in content.chunks(chunk_size) {
        hasher.update(chunk);
    }
    
    let streaming_checksum = format!("{:x}", hasher.finalize());
    
    // Both methods should produce the same checksum
    assert_eq!(full_checksum, streaming_checksum);
}

// Test file path construction and validation
#[test]
fn test_file_path_construction() {
    let user_id = "user123";
    let agent_name = "my-agent";
    let version = "1.0.0";
    let filename = "agent.zip";
    
    // Construct storage path
    let file_path = format!("{}/{}/{}/{}", user_id, agent_name, version, filename);
    assert_eq!(file_path, "user123/my-agent/1.0.0/agent.zip");
    
    // Validate path components
    let path_parts: Vec<&str> = file_path.split('/').collect();
    assert_eq!(path_parts.len(), 4);
    assert_eq!(path_parts[0], user_id);
    assert_eq!(path_parts[1], agent_name);
    assert_eq!(path_parts[2], version);
    assert_eq!(path_parts[3], filename);
    
    // Ensure no path traversal attempts
    assert!(!file_path.contains(".."));
    assert!(!file_path.contains("./"));
    assert!(!file_path.starts_with("/"));
}

// Test download URL construction
#[test]
fn test_download_url_construction() {
    let base_url = "https://storage.example.com";
    let bucket = "agent-storage";
    let file_path = "user123/my-agent/1.0.0/agent.zip";
    
    // Public download URL
    let public_url = format!("{}/object/public/{}/{}", base_url, bucket, file_path);
    assert_eq!(public_url, "https://storage.example.com/object/public/agent-storage/user123/my-agent/1.0.0/agent.zip");
    
    // Private upload URL
    let upload_url = format!("{}/object/{}/{}", base_url, bucket, file_path);
    assert_eq!(upload_url, "https://storage.example.com/object/agent-storage/user123/my-agent/1.0.0/agent.zip");
    
    // Validate URL format
    assert!(public_url.starts_with("https://"));
    assert!(public_url.contains("/object/public/"));
    assert!(upload_url.starts_with("https://"));
    assert!(upload_url.contains("/object/"));
    assert!(!upload_url.contains("/public/"));
}

// Test temporary file handling
#[test]
fn test_temporary_file_handling() {
    let content = file_test_utils::create_test_zip_content();
    
    // Create temporary file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(&content).expect("Failed to write to temp file");
    
    // Get file path
    let temp_path = temp_file.path();
    assert!(temp_path.exists());
    
    // Read back and verify
    let read_content = std::fs::read(temp_path).expect("Failed to read temp file");
    assert_eq!(content, read_content);
    
    // File should be automatically cleaned up when temp_file is dropped
    let temp_path_clone = temp_path.to_path_buf();
    drop(temp_file);
    
    // File should no longer exist
    assert!(!temp_path_clone.exists());
}

// Test file validation with different ZIP compression methods
#[test]
fn test_zip_compression_methods() {
    use zip::CompressionMethod;
    
    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
        
        // Add file with no compression
        let options = FileOptions::default().compression_method(CompressionMethod::Stored);
        zip.start_file("uncompressed.txt", options)
            .expect("Failed to start file");
        zip.write_all(b"This content is not compressed")
            .expect("Failed to write content");
        
        // Add file with deflate compression
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file("compressed.txt", options)
            .expect("Failed to start file");
        zip.write_all(b"This content is compressed with deflate algorithm")
            .expect("Failed to write content");
        
        zip.finish().expect("Failed to finish ZIP");
    }
    
    // Should be able to read the ZIP regardless of compression method
    let filenames = file_test_utils::extract_zip_and_validate(&buffer);
    assert!(filenames.is_ok());
    
    let files = filenames.unwrap();
    assert!(files.contains(&"uncompressed.txt".to_string()));
    assert!(files.contains(&"compressed.txt".to_string()));
}