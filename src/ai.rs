use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::model::{AiSettings, MacroPreset, MacroStep};

const API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Serialize)]
struct GenerateRequest<'a> {
    #[serde(rename = "systemInstruction")]
    system_instruction: Content<'a>,
    contents: [Content<'a>; 1],
}

#[derive(Serialize)]
struct Content<'a> {
    parts: [Part<'a>; 1],
}

#[derive(Serialize)]
struct Part<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct GenerateResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<ResponseContent>,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Option<Vec<ResponsePart>>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

pub fn generate_macro_steps(settings: &AiSettings, prompt: &str) -> Result<Vec<MacroStep>> {
    if settings.api_key.trim().is_empty() {
        bail!("Enter a Gemini API key first");
    }
    let model = settings.model.trim();
    if model.is_empty() {
        bail!("The AI model name is empty");
    }

    let url = format!("{API_URL}/{model}:generateContent?key={}", settings.api_key.trim());
    let body = GenerateRequest {
        system_instruction: Content {
            parts: [Part {
                text: settings.system_instruction.trim(),
            }],
        },
        contents: [Content {
            parts: [Part { text: prompt.trim() }],
        }],
    };

    let response = reqwest::blocking::Client::new()
        .post(url)
        .json(&body)
        .send()
        .context("Failed to call the Gemini API")?
        .error_for_status()
        .context("Gemini returned an error response")?;

    let payload: GenerateResponse = response
        .json()
        .context("Failed to parse the Gemini response")?;

    let text = payload
        .candidates
        .and_then(|mut list| list.drain(..).next())
        .and_then(|candidate| candidate.content)
        .and_then(|content| content.parts)
        .and_then(|mut parts| parts.drain(..).find_map(|part| part.text))
        .context("Gemini did not return any text")?;

    parse_steps_json(&text)
}

fn parse_steps_json(text: &str) -> Result<Vec<MacroStep>> {
    let trimmed = text.trim();
    let normalized = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|value| value.trim())
        .unwrap_or(trimmed);
    let normalized = normalized.strip_suffix("```").unwrap_or(normalized).trim();

    let steps: Vec<MacroStep> =
        serde_json::from_str(normalized).context("AI output was not valid macro JSON")?;
    Ok(steps)
}

pub fn apply_steps_to_preset(preset: &mut MacroPreset, steps: Vec<MacroStep>) {
    preset.steps = steps;
}
