use crate::error::AppError;
use crate::models::feed::Feed;
use crate::models::post::Post;
use crate::models::selector::PostContentSelector;
use crate::schema::feeds;
use crate::services::common::{get_gemini_request, parse_gemini_json_response};
use crate::services::selector::get_post_html;
use axum::extract::Path;
use axum::routing::get;
use axum::{Json, extract::State};
use chrono::Local;
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
    let selectors = parse_gemini_json_response::<PostContentSelector>(info).await?;

    let (title_html, content_html) = get_post_html(&html, &selectors).await?;
    let local_time = Local::now().naive_local();

    Ok(Json(Post {
        id: 0,
        feed_id: parent_feed_id,
        title: title_html,
        link: feed.link,
        content: content_html,
        created_at: local_time,
        updated_at: local_time,
    }))
}
