use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Instant;
use tokio_stream::StreamExt;
use virality_sim::{LlmScore, LlmTrace};

#[derive(Clone)]
pub struct LlmResult {
    pub score: LlmScore,
    pub trace: LlmTrace,
}

#[derive(Clone)]
pub struct LlmClient {
    client: reqwest::Client,
    api_key: String,
    api_base: String,
    model: String,
}

impl LlmClient {
    pub fn from_env(model_override: Option<String>) -> Option<Self> {
        let api_key = env::var("XAI_API_KEY").ok()?;
        let api_base = env::var("XAI_API_BASE").unwrap_or_else(|_| "https://api.x.ai/v1".to_string());
        let model = model_override
            .or_else(|| env::var("XAI_MODEL").ok())
            .unwrap_or_else(|| "grok-2-latest".to_string());
        let client = reqwest::Client::new();
        Some(Self {
            client,
            api_key,
            api_base,
            model,
        })
    }

    pub async fn score_text(&self, text: &str) -> Result<LlmResult, String> {
        let url = format!("{}/chat/completions", self.api_base.trim_end_matches('/'));
        let (messages, prompt) = build_messages(text);
        let request = ChatRequest {
            model: self.model.clone(),
            temperature: 0.2,
            messages,
            stream: None,
        };

        let started = Instant::now();
        let response = self
            .client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|err| format!("xAI request failed: {}", err))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| String::new());
            let detail = error_body.trim();
            if detail.is_empty() {
                return Err(format!("xAI API error: {}", status));
            }
            return Err(format!("xAI API error: {} {}", status, detail));
        }

        let body: ChatResponse = response
            .json()
            .await
            .map_err(|err| format!("xAI response parse failed: {}", err))?;

        let content = body
            .choices
            .first()
            .ok_or_else(|| "xAI response missing choices".to_string())?
            .message
            .content
            .trim()
            .to_string();

        let score = parse_score(&content)?;
        let usage = body.usage.unwrap_or_default();
        let trace = LlmTrace {
            model: body.model.unwrap_or_else(|| self.model.clone()),
            latency_ms: started.elapsed().as_millis(),
            prompt_summary: prompt_summary(),
            prompt,
            raw_response: content,
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        };

        Ok(LlmResult { score, trace })
    }

    pub async fn score_text_stream<F>(
        &self,
        text: &str,
        mut on_token: F,
    ) -> Result<LlmResult, String>
    where
        F: FnMut(&str) + Send,
    {
        let url = format!("{}/chat/completions", self.api_base.trim_end_matches('/'));
        let (messages, prompt) = build_messages(text);
        let request = ChatRequest {
            model: self.model.clone(),
            temperature: 0.2,
            messages,
            stream: Some(true),
        };

        let started = Instant::now();
        let response = self
            .client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|err| format!("xAI request failed: {}", err))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| String::new());
            let detail = error_body.trim();
            if detail.is_empty() {
                return Err(format!("xAI API error: {}", status));
            }
            return Err(format!("xAI API error: {} {}", status, detail));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut content = String::new();
        let mut model: Option<String> = None;
        let mut usage = ChatUsage::default();

        'outer: while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|err| format!("xAI stream failed: {}", err))?;
            let text_chunk = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text_chunk);
            if buffer.contains("\r\n") {
                buffer = buffer.replace("\r\n", "\n");
            }

            for data in drain_sse_events(&mut buffer) {
                if data == "[DONE]" {
                    break 'outer;
                }

                let event: ChatStreamResponse = serde_json::from_str(&data)
                    .map_err(|err| format!("xAI stream parse failed: {}", err))?;
                if let Some(model_value) = event.model {
                    model = Some(model_value);
                }
                if let Some(usage_value) = event.usage {
                    usage = usage_value;
                }
                if let Some(delta) = event
                    .choices
                    .first()
                    .and_then(|choice| choice.delta.content.as_deref())
                {
                    content.push_str(delta);
                    on_token(delta);
                }
                if event
                    .choices
                    .first()
                    .and_then(|choice| choice.finish_reason.as_deref())
                    == Some("stop")
                {
                    break 'outer;
                }
            }
        }

        let score = parse_score(&content)?;
        let trace = LlmTrace {
            model: model.unwrap_or_else(|| self.model.clone()),
            latency_ms: started.elapsed().as_millis(),
            prompt_summary: prompt_summary(),
            prompt,
            raw_response: content,
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        };

        Ok(LlmResult { score, trace })
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    model: Option<String>,
    usage: Option<ChatUsage>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Deserialize, Default)]
struct ChatUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct ChatStreamResponse {
    choices: Vec<ChatStreamChoice>,
    model: Option<String>,
    usage: Option<ChatUsage>,
}

#[derive(Deserialize)]
struct ChatStreamChoice {
    delta: ChatStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChatStreamDelta {
    content: Option<String>,
}

fn system_prompt() -> String {
    let prompt = r#"You are a strict JSON-only scorer for tweet virality signals.
Return a single JSON object with these fields:
- hook (0..1)
- clarity (0..1)
- novelty (0..1)
- shareability (0..1)
- controversy (0..1)
- sentiment (-1..1)
- suggestions (array of 3-5 short, actionable strings)
Rules:
- Output JSON only, no markdown or commentary.
- Use decimals with a leading 0 (e.g., 0.42).
"#;
    prompt.to_string()
}

fn prompt_summary() -> String {
    "Scores hook, clarity, novelty, shareability, controversy, sentiment + suggestions.".to_string()
}

fn build_messages(text: &str) -> (Vec<ChatMessage>, String) {
    let system = system_prompt();
    let user = format!("Tweet:\n{}", text);
    let prompt = format!("System:\n{}\n\nUser:\n{}", system, user);
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user,
        },
    ];
    (messages, prompt)
}

pub fn prompt_for_text(text: &str) -> String {
    let (_, prompt) = build_messages(text);
    prompt
}

fn parse_score(content: &str) -> Result<LlmScore, String> {
    let json = extract_json(content).ok_or_else(|| "xAI response missing JSON".to_string())?;
    let mut score: LlmScore = serde_json::from_str(&json)
        .map_err(|err| format!("xAI JSON parse failed: {}", err))?;

    score.hook = clamp01(score.hook);
    score.clarity = clamp01(score.clarity);
    score.novelty = clamp01(score.novelty);
    score.shareability = clamp01(score.shareability);
    score.controversy = clamp01(score.controversy);
    score.sentiment = score.sentiment.max(-1.0).min(1.0);
    score.suggestions = score
        .suggestions
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .take(6)
        .collect();

    Ok(score)
}

fn drain_sse_events(buffer: &mut String) -> Vec<String> {
    let mut events = Vec::new();
    loop {
        let Some(idx) = buffer.find("\n\n") else {
            break;
        };
        let block = buffer[..idx].to_string();
        *buffer = buffer[idx + 2..].to_string();
        let mut data_lines = Vec::new();
        for line in block.lines() {
            let line = line.trim_end_matches('\r');
            if let Some(data) = line.strip_prefix("data:") {
                data_lines.push(data.trim_start().to_string());
            }
        }
        if !data_lines.is_empty() {
            events.push(data_lines.join("\n"));
        }
    }
    events
}

fn extract_json(text: &str) -> Option<String> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if start >= end {
        return None;
    }
    Some(text[start..=end].to_string())
}

fn clamp01(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.max(0.0).min(1.0)
}
