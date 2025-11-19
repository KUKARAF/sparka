use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub created_time: DateTime<Utc>,
    pub modified_time: DateTime<Utc>,
    pub parents: Option<Vec<String>>,
    pub web_view_link: Option<String>,
    pub web_content_link: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriveFolder {
    pub id: String,
    pub name: String,
    pub created_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareRequest {
    pub role: String,
    pub type_: String,
    pub email_address: Option<String>,
    pub domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportRecord {
    pub file_id: String,
    pub file_name: String,
    pub imported_by: String,
    pub imported_at: DateTime<Utc>,
    pub calendar_event_id: Option<String>,
    pub status: String, // "imported", "failed", "pending"
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportLog {
    pub version: String,
    pub last_updated: DateTime<Utc>,
    pub imports: Vec<ImportRecord>,
}

impl ImportLog {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            last_updated: Utc::now(),
            imports: Vec::new(),
        }
    }
}

pub async fn create_or_get_sparka_folder(access_token: &str) -> Result<String> {
    let client = Client::new();
    
    // Check if Sparka folder exists
    let response = client
        .get("https://www.googleapis.com/drive/v3/files")
        .header("Authorization", format!("Bearer {}", access_token))
        .query(&[
            ("q", "name='Sparka' and mimeType='application/vnd.google-apps.folder' and trashed=false"),
            ("fields", "files(id,name)"),
        ])
        .send()
        .await?;
    
    let folder_response: DriveFileList = response.json().await?;
    
    if let Some(folder) = folder_response.files.first() {
        return Ok(folder.id.clone());
    }
    
    // Create new folder
    let folder_metadata = serde_json::json!({
        "name": "Sparka",
        "mimeType": "application/vnd.google-apps.folder"
    });
    
    let response = client
        .post("https://www.googleapis.com/drive/v3/files")
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&folder_metadata)
        .send()
        .await?;
    
    let folder: DriveFile = response.json().await?;
    Ok(folder.id)
}

pub async fn upload_file_to_drive(
    access_token: &str,
    folder_id: &str,
    file_name: &str,
    content: &[u8],
    mime_type: &str,
) -> Result<DriveFile> {
    let client = Client::new();
    
    // Create file metadata
    let metadata = serde_json::json!({
        "name": file_name,
        "parents": [folder_id],
        "mimeType": mime_type
    });
    
    // Multipart upload
    let form = reqwest::multipart::Form::new()
        .part("metadata", reqwest::multipart::Part::text(metadata.to_string())
            .file_name("metadata.json")
            .mime_str("application/json")?)
        .part("file", reqwest::multipart::Part::bytes(content.to_vec())
            .file_name(file_name)
            .mime_str(mime_type)?);
    
    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .header("Authorization", format!("Bearer {}", access_token))
        .multipart(form)
        .send()
        .await?;
    
    let uploaded_file: DriveFile = response.json().await?;
    Ok(uploaded_file)
}

pub async fn share_file_with_user(
    access_token: &str,
    file_id: &str,
    email: &str,
    role: &str, // "reader", "writer", "commenter"
) -> Result<()> {
    let client = Client::new();
    
    let share_request = ShareRequest {
        role: role.to_string(),
        type_: "user".to_string(),
        email_address: Some(email.to_string()),
        domain: None,
    };
    
    let response = client
        .post(&format!("https://www.googleapis.com/drive/v3/files/{}/permissions", file_id))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&share_request)
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to share file"))
    }
}

pub async fn share_folder_with_calendar_users(
    access_token: &str,
    folder_id: &str,
    calendar_users: &[String],
) -> Result<()> {
    for email in calendar_users {
        share_file_with_user(access_token, folder_id, email, "reader").await?;
    }
    Ok(())
}

pub async fn get_or_create_import_log(access_token: &str, folder_id: &str) -> Result<ImportLog> {
    let client = Client::new();
    
    // Look for import log file
    let response = client
        .get("https://www.googleapis.com/drive/v3/files")
        .header("Authorization", format!("Bearer {}", access_token))
        .query(&[
            ("q", &format!("name='import_log.json' and parents='{}' and trashed=false", folder_id)),
            ("fields", "files(id,name)"),
        ])
        .send()
        .await?;
    
    let file_list: DriveFileList = response.json().await?;
    
    if let Some(log_file) = file_list.files.first() {
        // Download existing log
        let response = client
            .get(&format!("https://www.googleapis.com/drive/v3/files/{}", log_file.id))
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[("alt", "media")])
            .send()
            .await?;
        
        let log_content = response.text().await?;
        let import_log: ImportLog = serde_json::from_str(&log_content)?;
        Ok(import_log)
    } else {
        // Create new log
        Ok(ImportLog::new())
    }
}

pub async fn update_import_log(
    access_token: &str,
    folder_id: &str,
    import_log: &ImportLog,
) -> Result<()> {
    let client = Client::new();
    
    let log_content = serde_json::to_string_pretty(import_log)?;
    let log_bytes = log_content.as_bytes();
    
    // Check if log file exists
    let response = client
        .get("https://www.googleapis.com/drive/v3/files")
        .header("Authorization", format!("Bearer {}", access_token))
        .query(&[
            ("q", &format!("name='import_log.json' and parents='{}' and trashed=false", folder_id)),
            ("fields", "files(id)"),
        ])
        .send()
        .await?;
    
    let file_list: DriveFileList = response.json().await?;
    
    if let Some(log_file) = file_list.files.first() {
        // Update existing file
        let response = client
            .patch(&format!("https://www.googleapis.com/upload/drive/v3/files/{}", log_file.id))
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[("uploadType", "media")])
            .body(log_bytes.to_vec())
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to update import log"));
        }
    } else {
        // Create new log file
        upload_file_to_drive(access_token, folder_id, "import_log.json", log_bytes, "application/json").await?;
    }
    
    Ok(())
}

pub async fn add_import_record(
    access_token: &str,
    folder_id: &str,
    file_id: &str,
    file_name: &str,
    imported_by: &str,
    calendar_event_id: Option<String>,
    status: &str,
    error_message: Option<String>,
) -> Result<()> {
    let mut import_log = get_or_create_import_log(access_token, folder_id).await?;
    
    let record = ImportRecord {
        file_id: file_id.to_string(),
        file_name: file_name.to_string(),
        imported_by: imported_by.to_string(),
        imported_at: Utc::now(),
        calendar_event_id,
        status: status.to_string(),
        error_message,
    };
    
    import_log.imports.push(record);
    import_log.last_updated = Utc::now();
    
    update_import_log(access_token, folder_id, &import_log).await?;
    Ok(())
}

pub async fn list_files_in_folder(access_token: &str, folder_id: &str) -> Result<Vec<DriveFile>> {
    let client = Client::new();
    
    let response = client
        .get("https://www.googleapis.com/drive/v3/files")
        .header("Authorization", format!("Bearer {}", access_token))
        .query(&[
            ("q", &format!("parents='{}' and trashed=false", folder_id)),
            ("fields", "files(id,name,mimeType,createdTime,modifiedTime,webViewLink,webContentLink)"),
        ])
        .send()
        .await?;
    
    let file_list: DriveFileList = response.json().await?;
    Ok(file_list.files)
}

#[derive(Debug, Deserialize)]
struct DriveFileList {
    files: Vec<DriveFile>,
}

pub async fn download_file_content(access_token: &str, file_id: &str) -> Result<Vec<u8>> {
    let client = Client::new();
    
    let response = client
        .get(&format!("https://www.googleapis.com/drive/v3/files/{}", file_id))
        .header("Authorization", format!("Bearer {}", access_token))
        .query(&[("alt", "media")])
        .send()
        .await?;
    
    Ok(response.bytes().await?.to_vec())
}