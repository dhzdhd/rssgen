use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    pub generation_config: GenerationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationConfig {
    // #[serde(rename = "thinkingConfig")]
    // pub thinking_config: ThinkingConfig,
    #[serde(rename = "responseMimeType")]
    pub response_mime_type: String,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct ThinkingConfig {
//     #[serde(rename = "thinkingBudget")]
//     pub thinking_budget: i32,
// }

pub type GeminiResponse = Vec<IndividualGeminiResponse>;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndividualGeminiResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub content: Content,
}
