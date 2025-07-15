use crate::models::feed::NewFeed;
use crate::models::post::PostSelector;
use crate::routes::post::posts_router;
use crate::schema::feeds::dsl::*;
use crate::services::common::get_html_selectors;
use crate::{error::AppError, models::feed::Feed};
use axum::extract::Path;
use axum::routing::{get, patch};
use axum::{Json, extract::State};
use chrono::NaiveDateTime;
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use diesel::upsert::excluded;
use scraper::{Html, Selector};

use axum::Router;

pub fn feeds_router() -> Router<Pool> {
    Router::new()
        .route("/", get(get_feeds).post(create_feed))
        .route("/scrape", get(analyze_url))
        .nest(
            "/{feed_id}",
            Router::new()
                .route("/", patch(update_feed).delete(delete_feed))
                .nest("/posts", posts_router()),
        )
}

async fn update_feed(
    State(pool): State<Pool>,
    Path(feed_id): Path<i32>,
) -> Result<Json<Feed>, AppError> {
    unimplemented!()
}

async fn delete_feed(
    State(pool): State<Pool>,
    Path(feed_id): Path<i32>,
) -> Result<Json<Feed>, AppError> {
    unimplemented!()
}

async fn get_feeds(State(pool): State<Pool>) -> Result<Json<Vec<Feed>>, AppError> {
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

async fn create_feed(
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

const FEED_SELECTOR_QUERY: &str = "Given this HTML, give me the title, author and a short description of this feed. The json you return should have three fields - title, author, description.\n\n";

#[axum::debug_handler]
async fn analyze_url() -> Result<Json<Feed>, AppError> {
    let response = reqwest::get("https://www.dhzdhd.dev/blog/gleam-executable").await?;
    let html = response.text().await?;

    let json = get_html_selectors::<PostSelector>(&html, FEED_SELECTOR_QUERY).await?;

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
        author: "".to_string(),
        description: Some(content_html),
        created_at: NaiveDateTime::MAX,
        updated_at: NaiveDateTime::MAX,
    }))
}
