use std::env;

use serde::Deserialize;

use crate::{
    error::AppError,
    models::gemini::{Content, GeminiRequest, GeminiResponse, GenerationConfig, Part},
};

pub async fn get_html_selectors<T: for<'a> Deserialize<'a>>(
    html: &str,
    query: &str,
) -> Result<T, AppError> {
    let info = get_gemini_request(html, query).await?;
    let raw_json = info
        .iter()
        .flat_map(|response| &response.candidates)
        .map(|candidate| &candidate.content)
        .flat_map(|content| &content.parts)
        .fold(String::new(), |acc, part| format!("{acc}{}", part.text));

    let selectors = serde_json::from_str::<T>(&raw_json)?;
    Ok(selectors)
}

pub async fn parse_gemini_json_response<T: for<'a> Deserialize<'a>>(
    info: GeminiResponse,
) -> Result<T, AppError> {
    let raw_json = info
        .iter()
        .flat_map(|response| &response.candidates)
        .map(|candidate| &candidate.content)
        .flat_map(|cont| &cont.parts)
        .fold(String::new(), |acc, part| format!("{acc}{}", part.text));

    let json = serde_json::from_str::<T>(&raw_json)?;
    Ok(json)
}

pub async fn get_gemini_request(html: &str, query: &str) -> Result<GeminiResponse, AppError> {
    let client = reqwest::Client::new();
    let body = GeminiRequest {
        contents: vec![Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: String::from_iter(vec![query, html]),
            }],
        }],
        generation_config: GenerationConfig {
            response_mime_type: "application/json".to_string(),
        },
    };

    let api_key = env::var("GEMINI_API_KEY")?;

    let response  = client.post(
        format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:streamGenerateContent?key={api_key}"),
    ).json(&body).send().await?.json::<GeminiResponse>().await?;

    Ok(response)
}
