use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::models::Settings;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Part {
    text: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct ResponsePart {
    text: String,
}

fn build_gemini_request(text: String, prompt: &str) -> GeminiRequest {
    GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: format!("{}\n\n{}", prompt, text),
            }],
        }],
    }
}

fn parse_gemini_response(resp: GeminiResponse) -> String {
    resp.candidates.first()
        .map(|c| c.content.parts.first().map(|p| p.text.clone()).unwrap_or_default())
        .unwrap_or_default()
}

pub async fn call_gemini(text: String, prompt: &str, settings: Settings) -> Result<String, String> {
    let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| "GOOGLE_API_KEY not found".to_string())?;
    let client = Client::new();
    
    let model = settings.model;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let request = build_gemini_request(text, prompt);

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
    
    Ok(parse_gemini_response(gemini_resp))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_gemini_request() {
        let req = build_gemini_request("world".to_string(), "hello");
        assert_eq!(req.contents.len(), 1);
        assert_eq!(req.contents[0].parts.len(), 1);
        assert_eq!(req.contents[0].parts[0].text, "hello\n\nworld");
    }

    #[test]
    fn test_parse_gemini_response() {
        let resp = GeminiResponse {
            candidates: vec![Candidate {
                content: ResponseContent {
                    parts: vec![ResponsePart {
                        text: "Mocked response".to_string(),
                    }],
                },
            }],
        };
        let result = parse_gemini_response(resp);
        assert_eq!(result, "Mocked response");
    }

    #[test]
    fn test_parse_empty_gemini_response() {
        let resp = GeminiResponse {
            candidates: vec![],
        };
        let result = parse_gemini_response(resp);
        assert_eq!(result, "");
    }
}
