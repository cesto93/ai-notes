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

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const OLLAMA_URL: &str = "http://localhost:11434/v1/chat/completions";
const LLAMACPP_URL: &str = "http://localhost:8080/v1/chat/completions";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct ResponseMessage {
    content: Option<String>,
}

fn build_openai_request(text: String, prompt: &str, model: String) -> OpenAIRequest {
    OpenAIRequest {
        model,
        messages: vec![Message {
            role: "user".to_string(),
            content: format!("{}\n\n{}", prompt, text),
        }],
    }
}

fn parse_openai_response(resp: OpenAIResponse) -> String {
    resp.choices.first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default()
}

async fn call_openai_compatible(
    url: &str,
    api_key_env: Option<&str>,
    text: String,
    prompt: &str,
    settings: Settings,
) -> Result<String, String> {
    let mut request = Client::new()
        .post(url)
        .json(&build_openai_request(text, prompt, settings.model));

    if let Some(env_var) = api_key_env {
        let api_key = std::env::var(env_var).map_err(|_| format!("{} not found", env_var))?;
        request = request.bearer_auth(api_key);
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("{} API error: {}", settings.provider, error_text));
    }

    let resp: OpenAIResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(parse_openai_response(resp))
}

pub async fn call_llm(text: String, prompt: &str, settings: Settings) -> Result<String, String> {
    match settings.provider.as_str() {
        "google" => call_gemini(text, prompt, settings).await,
        "openrouter" => call_openai_compatible(OPENROUTER_URL, Some("OPENROUTER_API_KEY"), text, prompt, settings).await,
        "ollama" => call_openai_compatible(OLLAMA_URL, None, text, prompt, settings).await,
        "llamacpp" => call_openai_compatible(LLAMACPP_URL, None, text, prompt, settings).await,
        other => Err(format!("Unsupported provider: {}", other)),
    }
}

pub async fn summarize(text: String, settings: Settings) -> Result<String, String> {
    call_llm(text, "Summarize the following note in a concise manner:", settings).await
}

pub async fn paraphrase(text: String, settings: Settings) -> Result<String, String> {
    call_llm(text, "Paraphrase the following note making it clearer:", settings).await
}

pub async fn generate_mindmap(text: String, settings: Settings) -> Result<String, String> {
    call_llm(text, "Create a Mermaid.js mindmap syntax for the following text. Only return the Mermaid syntax starting with 'mindmap' and nothing else. Do not use markdown code blocks. Ensure the syntax is valid for Mermaid.js.", settings).await
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

    #[test]
    fn test_build_openai_request() {
        let req = build_openai_request("world".to_string(), "hello", "openai/gpt-4o-mini".to_string());
        assert_eq!(req.model, "openai/gpt-4o-mini");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
        assert_eq!(req.messages[0].content, "hello\n\nworld");
    }

    #[test]
    fn test_parse_openai_response() {
        let resp = OpenAIResponse {
            choices: vec![Choice {
                message: ResponseMessage {
                    content: Some("Mocked response".to_string()),
                },
            }],
        };
        let result = parse_openai_response(resp);
        assert_eq!(result, "Mocked response");
    }

    #[test]
    fn test_parse_empty_openai_response() {
        let resp = OpenAIResponse {
            choices: vec![],
        };
        let result = parse_openai_response(resp);
        assert_eq!(result, "");
    }

    #[test]
    fn test_parse_openai_null_content() {
        let resp = OpenAIResponse {
            choices: vec![Choice {
                message: ResponseMessage {
                    content: None,
                },
            }],
        };
        let result = parse_openai_response(resp);
        assert_eq!(result, "");
    }
}
