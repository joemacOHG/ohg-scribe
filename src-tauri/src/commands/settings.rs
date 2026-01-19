// src-tauri/src/commands/settings.rs
// Commands for storing and retrieving app settings (including API key)

use log::{info, error};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("Failed to access settings directory: {0}")]
    DirectoryError(String),
    #[error("Failed to read settings: {0}")]
    ReadError(String),
    #[error("Failed to write settings: {0}")]
    WriteError(String),
}

impl serde::Serialize for SettingsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Serialize, Deserialize, Default)]
struct AppSettings {
    api_key: Option<String>,
    openai_key: Option<String>,
}

// Get the settings file path
fn get_settings_file(app: &AppHandle) -> Result<PathBuf, SettingsError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| SettingsError::DirectoryError(e.to_string()))?;
    
    // Create directory if it doesn't exist
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| SettingsError::DirectoryError(e.to_string()))?;
        info!("Created app data directory: {:?}", app_data_dir);
    }
    
    Ok(app_data_dir.join("settings.json"))
}

// Load settings from file
fn load_settings(app: &AppHandle) -> Result<AppSettings, SettingsError> {
    let settings_file = get_settings_file(app)?;
    
    if !settings_file.exists() {
        return Ok(AppSettings::default());
    }
    
    let content = fs::read_to_string(&settings_file)
        .map_err(|e| SettingsError::ReadError(e.to_string()))?;
    
    serde_json::from_str(&content)
        .map_err(|e| SettingsError::ReadError(e.to_string()))
}

// Save settings to file
fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), SettingsError> {
    let settings_file = get_settings_file(app)?;
    
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| SettingsError::WriteError(e.to_string()))?;
    
    fs::write(&settings_file, content)
        .map_err(|e| SettingsError::WriteError(e.to_string()))?;
    
    Ok(())
}

/// Get the stored AssemblyAI API key
#[tauri::command]
pub async fn get_api_key(app: AppHandle) -> Result<Option<String>, SettingsError> {
    info!("Loading API key from settings...");
    
    let settings = load_settings(&app)?;
    
    match &settings.api_key {
        Some(key) => {
            info!("API key found (length: {})", key.len());
            Ok(Some(key.clone()))
        },
        None => {
            info!("No API key found in settings");
            Ok(None)
        }
    }
}

/// Store the AssemblyAI API key
#[tauri::command]
pub async fn set_api_key(app: AppHandle, api_key: String) -> Result<(), SettingsError> {
    info!("Saving API key to settings (length: {})", api_key.len());
    
    let mut settings = load_settings(&app)?;
    settings.api_key = Some(api_key);
    save_settings(&app, &settings)?;
    
    info!("API key saved successfully");
    Ok(())
}

/// Delete the stored API key
#[tauri::command]
pub async fn delete_api_key(app: AppHandle) -> Result<(), SettingsError> {
    info!("Deleting API key from settings...");
    
    let mut settings = load_settings(&app)?;
    settings.api_key = None;
    save_settings(&app, &settings)?;
    
    info!("API key deleted");
    Ok(())
}

/// Get the stored OpenAI API key
#[tauri::command]
pub async fn get_openai_key(app: AppHandle) -> Result<Option<String>, SettingsError> {
    info!("Loading OpenAI API key from settings...");
    
    let settings = load_settings(&app)?;
    
    match &settings.openai_key {
        Some(key) => {
            info!("OpenAI API key found (length: {})", key.len());
            Ok(Some(key.clone()))
        },
        None => {
            info!("No OpenAI API key found in settings");
            Ok(None)
        }
    }
}

/// Store the OpenAI API key
#[tauri::command]
pub async fn set_openai_key(app: AppHandle, api_key: String) -> Result<(), SettingsError> {
    info!("Saving OpenAI API key to settings (length: {})", api_key.len());
    
    let mut settings = load_settings(&app)?;
    settings.openai_key = Some(api_key);
    save_settings(&app, &settings)?;
    
    info!("OpenAI API key saved successfully");
    Ok(())
}
