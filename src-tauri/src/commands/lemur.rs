// src-tauri/src/commands/lemur.rs
// LeMUR API integration for AI-powered speaker identification

use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LemurError {
    #[error("API request failed: {0}")]
    RequestFailed(String),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    #[error("API error: {0}")]
    ApiError(String),
}

impl serde::Serialize for LemurError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// LeMUR Question-Answer Request
#[derive(Serialize)]
struct LemurQuestion {
    question: String,
    answer_format: String,
}

#[derive(Serialize)]
struct LemurQARequest {
    questions: Vec<LemurQuestion>,
    input_text: String,
    context: String,
    final_model: String,
}

// LeMUR Response
#[derive(Deserialize)]
struct LemurQAResponseItem {
    question: String,
    answer: String,
}

#[derive(Deserialize)]
struct LemurQAResponse {
    response: Vec<LemurQAResponseItem>,
}

#[derive(Deserialize)]
struct LemurErrorResponse {
    error: String,
}

/// Identify speaker names from transcript using LeMUR
/// 
/// Takes the transcript text with speaker labels and returns a mapping
/// of speaker labels (A, B, C...) to inferred names.
#[tauri::command]
pub async fn identify_speakers(
    transcript_text: String,
    speaker_labels: Vec<String>,
    api_key: String,
) -> Result<HashMap<String, String>, LemurError> {
    info!("Identifying speakers for {} unique speakers", speaker_labels.len());
    
    let client = Client::new();
    
    // Build questions for each speaker
    let questions: Vec<LemurQuestion> = speaker_labels
        .iter()
        .map(|label| LemurQuestion {
            question: format!("Who is speaker {}?", label),
            answer_format: "<First Name> <Last Name (if available)>".to_string(),
        })
        .collect();
    
    let request_body = LemurQARequest {
        questions,
        input_text: transcript_text,
        context: "Your task is to infer the speaker's name from the speaker-labelled transcript. If a speaker introduces themselves or is addressed by name, use that. If you cannot determine a name, respond with 'Unknown'.".to_string(),
        final_model: "anthropic/claude-3-5-haiku-20241022".to_string(),
    };
    
    info!("Sending LeMUR request...");
    
    let response = client
        .post("https://api.assemblyai.com/lemur/v3/generate/question-answer")
        .header("Authorization", &api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| LemurError::RequestFailed(e.to_string()))?;
    
    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| LemurError::ParseError(e.to_string()))?;
    
    if !status.is_success() {
        // Try to parse error response
        if let Ok(error_response) = serde_json::from_str::<LemurErrorResponse>(&response_text) {
            return Err(LemurError::ApiError(error_response.error));
        }
        return Err(LemurError::ApiError(format!(
            "HTTP {}: {}",
            status,
            response_text
        )));
    }
    
    info!("Parsing LeMUR response...");
    
    let lemur_response: LemurQAResponse = serde_json::from_str(&response_text)
        .map_err(|e| LemurError::ParseError(format!("{}: {}", e, response_text)))?;
    
    // Build speaker mapping from response
    let mut speaker_mapping: HashMap<String, String> = HashMap::new();
    
    for qa in lemur_response.response {
        // Extract speaker label from question "Who is speaker X?"
        if let Some(label) = qa.question
            .strip_prefix("Who is speaker ")
            .and_then(|s| s.strip_suffix("?"))
        {
            let name = qa.answer.trim().to_string();
            // Only use the name if it's not "Unknown" and not empty
            if !name.is_empty() && name.to_lowercase() != "unknown" {
                speaker_mapping.insert(label.to_string(), name);
            }
        }
    }
    
    info!("Identified {} speakers: {:?}", speaker_mapping.len(), speaker_mapping);
    
    Ok(speaker_mapping)
}
