use std::env;

use crate::error::AppError;
use crate::models::feed::Feed;
use crate::models::gemini::{Content, GeminiRequest, GeminiResponse, GenerationConfig, Part};
use crate::models::post::{Post, PostSelector};
use crate::schema::feeds;
use crate::schema::posts::dsl::*;
use crate::services::common::get_gemini_request;
use axum::extract::Path;
use axum::routing::get;
use axum::{Json, extract::State};
use chrono::NaiveDateTime;
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;

use axum::Router;
use scraper::{Html, Selector};

pub fn posts_router() -> Router<Pool> {
    Router::new()
        .route("/", get(get_posts))
        .route("/scrape", get(scrape_posts))
}

async fn get_posts(
    State(pool): State<Pool>,
    Path(parent_feed_id): Path<i32>,
) -> Result<Json<Vec<Post>>, AppError> {
    let conn = pool.get().await?;

    conn.interact(move |conn| {
        let feed: Feed = feeds::table
            .filter(feeds::id.eq(parent_feed_id))
            .select(Feed::as_select())
            .get_result(conn)?;

        Post::belonging_to(&feed)
            .select(Post::as_select())
            .load(conn)
            .map(|posts_vec| Json(posts_vec))
            .map_err(|err| err.into())
    })
    .await
    .map_err(|err| AppError::from_str(err))?
}

const POST_SELECTOR_QUERY: &str = "Given this HTML, give me the CSS selector for the title and content of the blog post. The json you return should have two fields - title and content. It should be as general as possible while being accurate. It should not be linked to any framework or contain a unique ID\n\n";

#[axum::debug_handler]
async fn scrape_posts(
    State(pool): State<Pool>,
    Path(parent_feed_id): Path<i32>,
) -> Result<Json<Post>, AppError> {
    let conn = pool.get().await?;

    let feed: Feed = conn
        .interact(move |conn| {
            feeds::table
                .filter(feeds::id.eq(parent_feed_id))
                .select(Feed::as_select())
                .get_result(conn)
                .map_err(AppError::from_str)
        })
        .await
        .map_err(|err| AppError::from_str(err))??;

    let response = reqwest::get("https://www.dhzdhd.dev/blog/gleam-executable").await?;
    let html = response.text().await?;

    let info = get_gemini_request(&html, POST_SELECTOR_QUERY).await?;
    let raw_json = info
        .iter()
        .flat_map(|response| &response.candidates)
        .map(|candidate| &candidate.content)
        .flat_map(|cont| &cont.parts)
        .fold(String::new(), |acc, part| format!("{acc}{}", part.text));

    let json = serde_json::from_str::<PostSelector>(&raw_json)?;

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

    Ok(Json(Post {
        id: 0,
        feed_id: parent_feed_id,
        title: title_html,
        link: "".to_string(),
        content: content_html,
        created_at: NaiveDateTime::MAX,
        updated_at: NaiveDateTime::MAX,
    }))
}
