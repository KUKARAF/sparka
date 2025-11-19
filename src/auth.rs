use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleTokenInfo {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

// Store the access token received from Android AuthorizationClient
pub fn store_access_token(access_token: &str) -> Result<()> {
    // In a real implementation, you'd store this securely
    // For now, we'll just validate the token by getting user info
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        validate_token(access_token).await
    })
}

pub async fn validate_token(access_token: &str) -> Result<()> {
    let client = Client::new();
    
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    if response.status().is_success() {
        let _user_info: UserInfo = response.json().await?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid access token"))
    }
}

// Exchange server auth code for refresh token (backend operation)
pub async fn exchange_code_for_refresh_token(
    auth_code: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String> {
    let client = Client::new();
    
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("code", auth_code);
    params.insert("grant_type", "authorization_code");
    params.insert("redirect_uri", ""); // Empty for server-side flow
    
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await?;
    
    let token_info: GoogleTokenInfo = response.json().await?;
    
    token_info.refresh_token.ok_or_else(|| {
        anyhow::anyhow!("No refresh token received")
    })
}

// Use refresh token to get new access token (backend operation)
pub async fn refresh_access_token(
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String> {
    let client = Client::new();
    
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("refresh_token", refresh_token);
    params.insert("grant_type", "refresh_token");
    
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await?;
    
    let token_info: GoogleTokenInfo = response.json().await?;
    
    Ok(token_info.access_token)
}

// Revoke access token
pub async fn revoke_token(access_token: &str) -> Result<()> {
    let client = Client::new();
    
    let response = client
        .post(&format!("https://oauth2.googleapis.com/revoke?token={}", access_token))
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to revoke token"))
    }
}