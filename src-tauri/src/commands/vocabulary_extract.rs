use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    max_tokens: u32,
    response_format: ResponseFormat,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct ExtractedVocabulary {
    pub categories: Vec<ExtractedCategory>,
    pub suggested_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ExtractedCategory {
    pub name: String,
    pub terms: Vec<String>,
}

#[tauri::command]
pub async fn extract_document_text(path: String) -> Result<String, String> {
    let path = Path::new(&path);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or("Could not determine file type")?;

    match extension.as_str() {
        "txt" | "md" => {
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
        }
        "docx" => extract_docx_text(path),
        "pdf" => extract_pdf_text(path),
        _ => Err(format!("Unsupported file type: {}", extension)),
    }
}

fn extract_docx_text(path: &Path) -> Result<String, String> {
    use docx_rs::*;

    let file = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let docx = read_docx(&file).map_err(|e| format!("Failed to parse docx: {}", e))?;

    let mut text = String::new();
    for child in docx.document.children {
        if let DocumentChild::Paragraph(p) = child {
            for child in p.children {
                if let ParagraphChild::Run(run) = child {
                    for child in run.children {
                        if let RunChild::Text(t) = child {
                            text.push_str(&t.text);
                        }
                    }
                }
            }
            text.push('\n');
        }
    }

    Ok(text)
}

fn extract_pdf_text(path: &Path) -> Result<String, String> {
    // For PDF extraction, we'll try using pdftotext command if available
    use std::process::Command;

    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg(path)
        .arg("-")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8(out.stdout).map_err(|e| format!("Invalid UTF-8 in PDF: {}", e))
        }
        Ok(out) => Err(format!(
            "pdftotext failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )),
        Err(_) => Err("PDF extraction requires pdftotext (install poppler-utils). For now, please use .docx or .txt files.".to_string()),
    }
}

#[tauri::command]
pub async fn extract_vocabulary_terms(
    text: String,
    api_key: String,
) -> Result<ExtractedVocabulary, String> {
    let client = Client::new();

    // Truncate if too long (OpenAI has token limits)
    let truncated = if text.len() > 60000 {
        text[..60000].to_string()
    } else {
        text
    };

    let request = OpenAIRequest {
        model: "gpt-4o-mini".to_string(),
        max_tokens: 4096,
        response_format: ResponseFormat {
            format_type: "json_object".to_string(),
        },
        messages: vec![
            Message {
                role: "system".to_string(),
                content: EXTRACTION_PROMPT.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!(
                    "Extract domain-specific terms from this document:\n\n{}",
                    truncated
                ),
            },
        ],
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI API error {}: {}", status, body));
    }

    let response_body: OpenAIResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = response_body
        .choices
        .first()
        .ok_or("No response from model")?
        .message
        .content
        .clone();

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse terms: {}", e))
}

const EXTRACTION_PROMPT: &str = r#"You extract domain-specific terms from documents to improve speech-to-text accuracy.

Extract terms in these categories:
1. Drug Names — Generic names, brand names, drug classes
2. Medical Terms — Conditions, procedures, biomarkers
3. Acronyms — Medical and business abbreviations  
4. Industry Terms — Specialized terminology
5. Organizations — Company names, institutions

Guidelines:
- Focus on terms speech-to-text might misrecognize
- Include multi-word phrases (up to 6 words)
- Exclude common words like "patient", "treatment"
- Prioritize proper nouns, acronyms, drug names

Return JSON:
{
  "categories": [
    {"name": "Drug Names", "terms": ["term1", "term2"]},
    {"name": "Medical Terms", "terms": [...]}
  ],
  "suggested_name": "Name based on document content"
}

Only return valid JSON. Omit empty categories. Aim for 20-150 terms."#;
