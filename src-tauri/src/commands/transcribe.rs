use log::{info, error};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

const ASSEMBLYAI_API_BASE: &str = "https://api.assemblyai.com/v2";

#[derive(Error, Debug)]
pub enum TranscribeError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("File error: {0}")]
    FileError(String),
    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),
    #[error("Timeout waiting for transcription")]
    Timeout,
}

impl serde::Serialize for TranscribeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Options for transcription request
#[derive(Debug, Deserialize)]
pub struct TranscriptionOptions {
    pub max_speakers: Option<i32>, // None = auto (defaults to 1-10), Some(n) = max n speakers
    pub boost_words: Vec<String>,
    pub include_summary: bool,
    pub detect_topics: bool,
    pub analyze_sentiment: bool,
    pub extract_key_phrases: bool,  // auto_highlights in AssemblyAI
    pub speaker_label_mode: String,  // 'generic' | 'auto-names' | 'known-names' | 'interview' | etc.
    pub speaker_values: Vec<String>,  // User-provided names or custom roles
}

/// Response from upload endpoint
#[derive(Debug, Deserialize)]
struct UploadResponse {
    upload_url: String,
}

/// Speaker identification configuration
#[derive(Debug, Serialize, Clone)]
struct SpeakerIdentification {
    speaker_type: String,  // "role" or "name"
    known_values: Vec<String>,
}

/// Speech understanding request wrapper
#[derive(Debug, Serialize, Clone)]
struct SpeechUnderstandingRequest {
    speaker_identification: SpeakerIdentification,
}

/// Speech understanding wrapper
#[derive(Debug, Serialize, Clone)]
struct SpeechUnderstanding {
    request: SpeechUnderstandingRequest,
}

/// Request body for transcript submission
#[derive(Debug, Serialize)]
struct TranscriptRequest {
    audio_url: String,
    speaker_labels: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    speakers_expected: Option<i32>,  // Used when exact count known
    #[serde(skip_serializing_if = "Vec::is_empty")]
    word_boost: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    summarization: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary_type: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    iab_categories: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    sentiment_analysis: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    auto_highlights: bool,  // Key phrases extraction
    #[serde(skip_serializing_if = "Option::is_none")]
    speech_understanding: Option<SpeechUnderstanding>,
}

/// Response from transcript submission and polling
/// Using flatten to capture any extra fields we don't explicitly handle
#[derive(Debug, Deserialize, Serialize)]
pub struct TranscriptResponse {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub utterances: Option<Vec<Utterance>>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub iab_categories_result: Option<IabCategoriesResult>,
    #[serde(default)]
    pub sentiment_analysis_results: Option<Vec<SentimentResult>>,
    #[serde(default)]
    pub error: Option<String>,
    // Capture any other fields we don't explicitly handle
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Utterance {
    pub speaker: String,
    pub text: String,
    pub start: i64,
    pub end: i64,
    #[serde(default)]
    pub words: Vec<Word>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Word {
    pub text: String,
    pub start: i64,
    pub end: i64,
    #[serde(default)]
    pub speaker: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IabCategoriesResult {
    #[serde(default)]
    pub summary: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SentimentResult {
    pub text: String,
    pub start: i64,
    pub end: i64,
    pub sentiment: String, // "POSITIVE", "NEGATIVE", "NEUTRAL"
    pub confidence: f64,
    #[serde(default)]
    pub speaker: Option<String>,
}

/// Upload an audio file to AssemblyAI and return the upload URL
/// AssemblyAI expects the raw audio bytes in the request body, not multipart form
#[tauri::command]
pub async fn upload_audio(file_path: String, api_key: String) -> Result<String, TranscribeError> {
    info!("Starting upload for file: {}", file_path);
    
    let path = PathBuf::from(&file_path);
    
    // Check if file exists
    if !path.exists() {
        error!("File does not exist: {}", file_path);
        return Err(TranscribeError::FileError(format!("File does not exist: {}", file_path)));
    }
    
    // Read the file
    info!("Opening file...");
    let mut file = File::open(&path)
        .await
        .map_err(|e| {
            error!("Failed to open file: {}", e);
            TranscribeError::FileError(format!("Failed to open file: {}", e))
        })?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .map_err(|e| {
            error!("Failed to read file: {}", e);
            TranscribeError::FileError(format!("Failed to read file: {}", e))
        })?;
    
    info!("File read successfully, size: {} bytes", buffer.len());
    
    // Log API key presence (not the actual key!)
    info!("API key present: {}, length: {}", !api_key.is_empty(), api_key.len());
    
    // AssemblyAI upload endpoint expects raw bytes in body with Content-Type header
    info!("Uploading to AssemblyAI...");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/upload", ASSEMBLYAI_API_BASE))
        .header("Authorization", &api_key)
        .header("Content-Type", "application/octet-stream")
        .body(buffer)
        .send()
        .await
        .map_err(|e| {
            error!("Upload request failed: {}", e);
            TranscribeError::RequestFailed(format!("Upload request failed: {}", e))
        })?;
    
    let status = response.status();
    info!("Upload response status: {}", status);
    
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        error!("Upload failed with status {}: {}", status, text);
        return Err(TranscribeError::ApiError(format!(
            "Upload failed with status {}: {}",
            status, text
        )));
    }
    
    let upload_response: UploadResponse = response
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse upload response: {}", e);
            TranscribeError::RequestFailed(format!("Failed to parse upload response: {}", e))
        })?;
    
    info!("Upload successful! URL: {}", upload_response.upload_url);
    Ok(upload_response.upload_url)
}

/// Submit a transcription job to AssemblyAI
#[tauri::command]
pub async fn submit_transcription(
    upload_url: String,
    api_key: String,
    options: TranscriptionOptions,
) -> Result<String, TranscribeError> {
    info!("Submitting transcription for: {}", upload_url);
    
    // Build speech_understanding config based on speaker label mode
    let speech_understanding = match options.speaker_label_mode.as_str() {
        "auto-names" => Some(SpeechUnderstanding {
            request: SpeechUnderstandingRequest {
                speaker_identification: SpeakerIdentification {
                    speaker_type: "name".to_string(),
                    known_values: vec![],  // Empty = auto-detect names from conversation
                },
            },
        }),
        "known-names" => Some(SpeechUnderstanding {
            request: SpeechUnderstandingRequest {
                speaker_identification: SpeakerIdentification {
                    speaker_type: "name".to_string(),
                    known_values: options.speaker_values.clone(),
                },
            },
        }),
        "interview" => Some(SpeechUnderstanding {
            request: SpeechUnderstandingRequest {
                speaker_identification: SpeakerIdentification {
                    speaker_type: "role".to_string(),
                    known_values: vec!["Interviewer".to_string(), "Interviewee".to_string()],
                },
            },
        }),
        "podcast" => Some(SpeechUnderstanding {
            request: SpeechUnderstandingRequest {
                speaker_identification: SpeakerIdentification {
                    speaker_type: "role".to_string(),
                    known_values: vec!["Host".to_string(), "Guest".to_string()],
                },
            },
        }),
        "panel" => Some(SpeechUnderstanding {
            request: SpeechUnderstandingRequest {
                speaker_identification: SpeakerIdentification {
                    speaker_type: "role".to_string(),
                    known_values: vec!["Moderator".to_string(), "Panelist".to_string()],
                },
            },
        }),
        "custom-roles" => Some(SpeechUnderstanding {
            request: SpeechUnderstandingRequest {
                speaker_identification: SpeakerIdentification {
                    speaker_type: "role".to_string(),
                    known_values: options.speaker_values.clone(),
                },
            },
        }),
        _ => None,  // "generic" or unrecognized - no speaker identification
    };
    
    if speech_understanding.is_some() {
        info!("Speaker identification enabled with mode: {:?}", options.speaker_label_mode);
    }
    
    let mut request = TranscriptRequest {
        audio_url: upload_url,
        speaker_labels: true,
        speakers_expected: options.max_speakers,  // Use as hint for max speakers
        word_boost: options.boost_words,
        summarization: options.include_summary,
        summary_model: if options.include_summary { Some("informative".to_string()) } else { None },
        summary_type: if options.include_summary { Some("bullets".to_string()) } else { None },
        iab_categories: options.detect_topics,
        sentiment_analysis: options.analyze_sentiment,
        auto_highlights: options.extract_key_phrases,
        speech_understanding,
    };
    
    // Ensure word_boost doesn't exceed limits
    if request.word_boost.len() > 200 {
        request.word_boost.truncate(200);
    }
    
    info!("Request: {:?}", request);
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/transcript", ASSEMBLYAI_API_BASE))
        .header("Authorization", &api_key)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            error!("Transcription request failed: {}", e);
            TranscribeError::RequestFailed(format!("Transcription request failed: {}", e))
        })?;
    
    let status = response.status();
    info!("Transcription submit response status: {}", status);
    
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        error!("Transcription submission failed with status {}: {}", status, text);
        return Err(TranscribeError::ApiError(format!(
            "Transcription submission failed with status {}: {}",
            status, text
        )));
    }
    
    // Get raw response text first for debugging
    let response_text = response.text().await.map_err(|e| {
        error!("Failed to get response text: {}", e);
        TranscribeError::RequestFailed(format!("Failed to get response text: {}", e))
    })?;
    
    info!("Raw response (first 500 chars): {}", &response_text.chars().take(500).collect::<String>());
    
    // Parse the response
    let transcript_response: TranscriptResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            error!("Failed to parse transcript response: {}. Response was: {}", e, &response_text.chars().take(1000).collect::<String>());
            TranscribeError::RequestFailed(format!("Failed to parse transcript response: {}", e))
        })?;
    
    info!("Transcription submitted! ID: {}", transcript_response.id);
    Ok(transcript_response.id)
}

/// Poll for transcription completion
#[tauri::command]
pub async fn poll_transcription(
    transcript_id: String,
    api_key: String,
) -> Result<TranscriptResponse, TranscribeError> {
    info!("Polling transcription: {}", transcript_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/transcript/{}", ASSEMBLYAI_API_BASE, transcript_id))
        .header("Authorization", &api_key)
        .send()
        .await
        .map_err(|e| {
            error!("Poll request failed: {}", e);
            TranscribeError::RequestFailed(format!("Poll request failed: {}", e))
        })?;
    
    let status = response.status();
    
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        error!("Poll request failed with status {}: {}", status, text);
        return Err(TranscribeError::ApiError(format!(
            "Poll request failed with status {}: {}",
            status, text
        )));
    }
    
    // Get raw response text first for debugging
    let response_text = response.text().await.map_err(|e| {
        error!("Failed to get poll response text: {}", e);
        TranscribeError::RequestFailed(format!("Failed to get poll response text: {}", e))
    })?;
    
    // Parse the response
    let transcript_response: TranscriptResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            error!("Failed to parse poll response: {}. Response was: {}", e, &response_text.chars().take(1000).collect::<String>());
            TranscribeError::RequestFailed(format!("Failed to parse poll response: {}", e))
        })?;
    
    info!("Poll result - Status: {}", transcript_response.status);
    
    // Check for error status
    if transcript_response.status == "error" {
        let err_msg = transcript_response.error.clone().unwrap_or_else(|| "Unknown error".to_string());
        error!("Transcription failed: {}", err_msg);
        return Err(TranscribeError::TranscriptionFailed(err_msg));
    }
    
    Ok(transcript_response)
}
