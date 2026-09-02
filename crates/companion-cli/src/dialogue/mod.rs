pub mod deterministic;
pub mod ollama;

use companion_types::PetState;
use reqwest::Client;
use tracing::warn;

#[allow(dead_code)]
pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434/api/generate";
#[allow(dead_code)]
pub const DEFAULT_OLLAMA_MODEL: &str = "qwen2.5-coder:0.5b";

#[allow(dead_code)]
pub async fn generate_dialogue(
    client: &Client,
    state: PetState,
    context: &str,
) -> String {
    match ollama::generate_ollama_dialogue(
        client,
        DEFAULT_OLLAMA_ENDPOINT,
        DEFAULT_OLLAMA_MODEL,
        state,
        context,
    )
    .await
    {
        Ok(dialogue) => dialogue,
        Err(err) => {
            warn!("Ollama dialogue failed/timed out: {}. Using deterministic fallback.", err);
            deterministic::get_deterministic_dialogue(state, context)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_dialogue_fallback() {
        let client = Client::new();
        let res = generate_dialogue(&client, PetState::Distressed, "HighCPU").await;
        assert_eq!(res, "Fans are screaming! What are we compiling?");
    }
}
