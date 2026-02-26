use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::models::Settings;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

pub async fn call_gemini(text: String, prompt: &str, settings: Settings) -> Result<String, String> {
    let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| "GOOGLE_API_KEY not found".to_string())?;
    let client = Client::new();
    
    let model = settings.model;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let request = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: format!("{}\n\n{}", prompt, text),
            }],
        }],
    };

    let response = client.post(url)
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Gemini API error: {}", error_text));
    }

    let gemini_resp: GeminiResponse = response.json().await.map_err(|e| e.to_string())?;
    
    let result = gemini_resp.candidates.first()
        .map(|c| c.content.parts.first().map(|p| p.text.clone()).unwrap_or_default())
        .unwrap_or_default();

    Ok(result)
}

pub async fn summarize(text: String, settings: Settings) -> Result<String, String> {
    call_gemini(text, "Summarize the following note in a concise manner:", settings).await
}

pub async fn paraphrase(text: String, settings: Settings) -> Result<String, String> {
    call_gemini(text, "Paraphrase the following note making it clearer:", settings).await
}

pub async fn generate_mindmap(text: String, settings: Settings) -> Result<String, String> {
    call_gemini(text, "Create a Mermaid.js mindmap syntax for the following text. Only return the Mermaid syntax starting with 'mindmap' and nothing else. Do not use markdown code blocks. Ensure the syntax is valid for Mermaid.js.", settings).await
}
