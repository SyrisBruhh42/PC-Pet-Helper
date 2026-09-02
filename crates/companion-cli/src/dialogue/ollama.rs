use companion_types::PetState;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    system: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[allow(dead_code)]
pub async fn generate_ollama_dialogue(
    client: &Client,
    endpoint: &str,
    model: &str,
    state: PetState,
    context: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = format!("Context: {}, State: {:?}. Give a one-line comment.", context, state);
    let request_body = OllamaRequest {
        model: model.to_string(),
        system: "You are Pixel, a 16-bit desktop cat companion. Max 10 words. Snarky but loyal."
            .to_string(),
        prompt,
        stream: false,
    };

    debug!("Sending dialogue request to Ollama endpoint: {}", endpoint);
    let res = client
        .post(endpoint)
        .json(&request_body)
        .timeout(Duration::from_millis(1500))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Ollama returned non-success status: {}", res.status()).into());
    }

    let response_body: OllamaResponse = res.json().await?;
    let text = response_body.response.trim().to_string();
    if text.is_empty() {
        return Err("Ollama returned empty response".into());
    }

    Ok(text)
}
