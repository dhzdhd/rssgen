use std::env;

use crate::models::feed::{FeedContent, NewFeed};
use crate::models::gemini::{Content, GeminiRequest, GeminiResponse, GenerationConfig, Part};
use crate::schema::feeds::dsl::*;
use crate::{error::AppError, models::feed::Feed};
use axum::{Json, extract::State};
use chrono::NaiveDateTime;
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use diesel::upsert::excluded;
use scraper::{Html, Selector};

pub async fn get_feeds(State(pool): State<Pool>) -> Result<Json<Vec<Feed>>, AppError> {
    let conn = pool.get().await?;

    conn.interact(|conn| {
        feeds
            .select(Feed::as_select())
            .load(conn)
            .map(|feeds_vec| Json(feeds_vec))
            .map_err(|err| err.into())
    })
    .await
    .map_err(|err| AppError::from_str(err))?
}

pub async fn create_feed(
    State(pool): State<Pool>,
    Json(new_feed): Json<NewFeed>,
) -> Result<Json<Feed>, AppError> {
    let conn = pool.get().await?;

    conn.interact(|conn: &mut PgConnection| {
        diesel::insert_into(feeds)
            .values(new_feed)
            .on_conflict(link)
            .do_update()
            .set((
                title.eq(excluded(title)),
                description.eq(excluded(description)),
                updated_at.eq(diesel::dsl::now),
            ))
            .get_result::<Feed>(conn)
            .map(|feed| feed)
    })
    .await
    .map_err(|err| AppError::from_str(err))?
    .map(Json)
    .map_err(|err| err.into())
}

#[axum::debug_handler]
pub async fn analyze_url() -> Result<Json<Feed>, AppError> {
    let response = reqwest::get("https://www.dhzdhd.dev/blog/gleam-executable").await?;
    let html = response.text().await?;

    let info = get_post_info(&html).await?;
    let raw_json = info
        .iter()
        .flat_map(|response| &response.candidates)
        .map(|candidate| &candidate.content)
        .flat_map(|content| &content.parts)
        .fold(String::new(), |acc, part| format!("{acc}{}", part.text));

    let json = serde_json::from_str::<FeedContent>(&raw_json)?;

    let (title_html, content_html) = tokio::task::spawn_blocking(move || {
        let parsed = Html::parse_document(&html);
        let title_selector = Selector::parse(&json.title).map_err(AppError::from_str)?;
        let content_selector = Selector::parse(&json.content).map_err(AppError::from_str)?;

        let title_html = parsed.select(&title_selector).next().unwrap().inner_html();
        let content_html = parsed
            .select(&content_selector)
            .next()
            .unwrap()
            .inner_html();

        Ok::<(String, String), AppError>((title_html, content_html))
    })
    .await??;

    Ok(Json(Feed {
        id: 0,
        title: title_html,
        link: "".to_string(),
        description: Some(content_html),
        created_at: NaiveDateTime::MAX,
        updated_at: NaiveDateTime::MAX,
    }))
}

pub async fn get_post_info(html: &str) -> Result<GeminiResponse, AppError> {
    let client = reqwest::Client::new();
    let body = GeminiRequest {
        contents: vec![Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: format!(
                    "Given this HTML, give me the CSS selector for the title and content of the blog post. The json you return should have two fields - title and content. It should be as general as possible while being accurate. It should not be linked to any framework or contain a unique ID\n\n{}",
                    html
                ),
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
