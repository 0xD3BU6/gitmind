use crate::config::Settings;
use crate::error::Result;
use rig_core::client::CompletionClient;
use rig_core::completion::{AssistantContent, CompletionModel};
use rig_core::providers::groq;
use std::sync::mpsc::{Receiver, channel};

/// Thin wrapper over the provider's completion model.
pub struct Client {
    api_key: String,
    model: String,
}

impl Client {
    pub fn new(settings: &Settings) -> Result<Self> {
        if !settings.has_ai() {
            return Err("no LLM API key configured — press ',' to open settings".into());
        }
        Ok(Self {
            api_key: settings.llm_api_key.trim().to_string(),
            model: settings.model.trim().to_string(),
        })
    }

    /// Send one prompt and return the text of the reply.
    pub async fn ask(&self, preamble: &str, prompt: &str) -> Result<String> {
        let client = groq::Client::new(self.api_key.as_str())?;
        let model = client.completion_model(self.model.as_str());
        let request = model
            .completion_request(prompt)
            .preamble(preamble.to_string())
            .temperature(0.3)
            .max_tokens(2000)
            .build();
        let response = model.completion(request).await?;
        let mut out = String::new();
        for item in response.choice {
            if let AssistantContent::Text(t) = item {
                out.push_str(&t.text);
            }
        }
        if out.trim().is_empty() {
            return Err("the model returned an empty reply".into());
        }
        Ok(out)
    }

    /// Blocking helper: run `ask` on a private single-thread runtime.
    pub fn ask_blocking(&self, preamble: &str, prompt: &str) -> Result<String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(self.ask(preamble, prompt))
    }
}

/// Generate a commit message for `diff` on a background thread.
/// The receiver yields exactly one message: the proposal or an error string.
pub fn spawn_commit_message(settings: Settings, diff: String, summary: String) -> Receiver<std::result::Result<String, String>> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let result = (|| -> Result<String> {
            let client = Client::new(&settings)?;
            let prompt = crate::ai::prompts::commit_prompt(&diff, &summary);
            let raw = client.ask_blocking(crate::ai::prompts::COMMIT_PREAMBLE, &prompt)?;
            let proposal = crate::ai::parse_proposal(&raw);
            Ok(proposal.to_message())
        })();
        let _ = tx.send(result.map_err(|e| e.to_string()));
    });
    rx
}
