use anyhow::Result;
use std::fs;
use std::path::Path;
use mime_guess::from_path;

pub fn read_file_from_uri(uri: &str) -> Result<String> {
    // Handle different URI formats
    let file_path = if uri.starts_with("file://") {
        uri.strip_prefix("file://").unwrap_or(uri)
    } else if uri.starts_with("content://") {
        // For Android content URIs, we'd need to use ContentResolver
        // For now, return an error that should be handled in Android code
        return Err(anyhow::anyhow!("Content URI handling requires Android ContentResolver"));
    } else {
        uri
    };
    
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", file_path));
    }
    
    let mime_type = from_path(path);
    
    match mime_type.type_().as_str() {
        "text" | "application" => {
            // Try to read as text first
            match fs::read_to_string(path) {
                Ok(content) => Ok(content),
                Err(_) => {
                    // If text reading fails, try binary and convert to base64
                    let bytes = fs::read(path)?;
                    Ok(format!("Binary file ({}): {}", mime_type, base64::encode(&bytes)))
                }
            }
        }
        "image" => {
            // For images, we could implement OCR in the future
            let bytes = fs::read(path)?;
            Ok(format!("Image file ({}): {}", mime_type, base64::encode(&bytes)))
        }
        _ => {
            let bytes = fs::read(path)?;
            Ok(format!("Binary file ({}): {}", mime_type, base64::encode(&bytes)))
        }
    }
}

pub fn extract_filename_from_uri(uri: &str) -> Result<String> {
    let file_path = if uri.starts_with("file://") {
        uri.strip_prefix("file://").unwrap_or(uri)
    } else if uri.starts_with("content://") {
        // Extract filename from content URI
        uri.split('/').last().unwrap_or("unknown_file").to_string()
    } else {
        uri
    };
    
    let path = Path::new(file_path);
    Ok(path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown_file")
        .to_string())
}

pub fn get_mime_type(uri: &str) -> Result<String> {
    let file_path = if uri.starts_with("file://") {
        uri.strip_prefix("file://").unwrap_or(uri)
    } else {
        uri
    };
    
    let path = Path::new(file_path);
    let mime_type = from_path(path);
    Ok(mime_type.first_or_octet_stream().to_string())
}

// Add base64 dependency to Cargo.toml if needed
pub fn is_supported_file_type(uri: &str) -> bool {
    let path = Path::new(uri);
    let mime_type = from_path(path);
    
    let supported_types = vec![
        "text/plain",
        "text/pdf",
        "application/pdf",
        "image/jpeg",
        "image/png",
        "image/gif",
    ];
    
    supported_types.contains(&mime_type.first_or_octet_stream().as_str())
}
    
    let mime_type = from_path(path).first_or_octet_stream();
    
    match mime_type.type_().as_str() {
        "text" | "application" => {
            // Try to read as text first
            match fs::read_to_string(path) {
                Ok(content) => Ok(content),
                Err(_) => {
                    // If text reading fails, try binary and convert to base64
                    let bytes = fs::read(path)?;
                    Ok(format!("Binary file ({}): {}", mime_type, base64::encode(&bytes)))
                }
            }
        }
        "image" => {
            // For images, we could implement OCR in the future
            let bytes = fs::read(path)?;
            Ok(format!("Image file ({}): {}", mime_type, base64::encode(&bytes)))
        }
        _ => {
            let bytes = fs::read(path)?;
            Ok(format!("Binary file ({}): {}", mime_type, base64::encode(&bytes)))
        }
    }
}

// Add base64 dependency to Cargo.toml if needed
pub fn is_supported_file_type(uri: &str) -> bool {
    let path = Path::new(uri);
    let mime_type = from_path(path);
    
    let supported_types = vec![
        "text/plain",
        "text/pdf",
        "application/pdf",
        "image/jpeg",
        "image/png",
        "image/gif",
    ];
    
    supported_types.contains(&mime_type.first_or_octet_stream().as_str())
}