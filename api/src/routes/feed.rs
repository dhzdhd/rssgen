use crate::models::feed::NewFeed;
use crate::models::selector::{FeedContent, FeedContentResponse};
use crate::routes::post::posts_router;
use crate::schema::feeds::dsl::*;
use crate::services::common::get_html_selectors;
use crate::{error::AppError, models::feed::Feed};
use axum::extract::{Path, Query};
use axum::routing::{get, patch};
use axum::{Json, extract::State};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use diesel::upsert::excluded;
use scraper::{ElementRef, Html, Selector};

use axum::Router;
use serde::Deserialize;

pub fn feeds_router() -> Router<Pool> {
    Router::new()
        .route("/", get(get_feeds).post(create_feed))
        .route("/scrape", get(scrape_feed))
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
    Json(new_feed): Json<NewFeed>,
) -> Result<Json<Feed>, AppError> {
    let conn = pool.get().await?;

    conn.interact(move |conn: &mut PgConnection| {
        diesel::update(feeds.filter(id.eq(feed_id)))
            .set((
                title.eq(&new_feed.title),
                description.eq(&new_feed.description),
                author.eq(&new_feed.author),
                updated_at.eq(diesel::dsl::now),
            ))
            .get_result::<Feed>(conn)
    })
    .await
    .map_err(|err| AppError::from_str(err))?
    .map(Json)
    .map_err(|err| err.into())
}

async fn delete_feed(
    State(pool): State<Pool>,
    Path(feed_id): Path<i32>,
) -> Result<Json<Feed>, AppError> {
    let conn = pool.get().await?;

    conn.interact(move |conn| diesel::delete(feeds.filter(id.eq(feed_id))).get_result(conn))
        .await
        .map_err(|err| AppError::from_str(err))?
        .map(Json)
        .map_err(|err| err.into())
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

const FEED_SELECTOR_QUERY: &str = "Given this HTML, give me the title, author, a short description, the CSS selector for the HTML element containing all the posts and the CSS selector for the HTML element containing the link for a post of this feed (an a element). Generate your own title and description if it does not exist. Return null if author does not exist. The json you return should have five fields - title, author, description, post_list_element, post_link_element.\n\n";

#[derive(Deserialize)]
struct ScrapeQuery {
    url: String,
}

async fn get_post_links(
    url: String,
    post_link_selector: &Selector,
    next_page_selector: &Selector,
) -> Result<(Vec<Option<String>>, Option<String>), AppError> {
    let response = reqwest::get(url).await?;
    let html = response.text().await?;
    let parsed_page = Html::parse_document(&html);

    let post_links = parsed_page
        .select(&post_link_selector)
        .collect::<Vec<ElementRef>>()
        .iter()
        .map(|item| item.attr("href").map(|item| item.to_string()))
        .collect::<Vec<Option<String>>>();

    let next_page_link = parsed_page
        .select(&next_page_selector)
        .next()
        .unwrap()
        .attr("href")
        .map(|item| item.to_string());

    Ok((post_links, next_page_link))
}

#[axum::debug_handler]
async fn scrape_feed(Query(params): Query<ScrapeQuery>) -> Result<Json<FeedContent>, AppError> {
    let response = reqwest::get(&params.url).await?;
    let html = response.text().await?;

    let json = get_html_selectors::<FeedContentResponse>(&html, FEED_SELECTOR_QUERY).await?;

    let next_page_selector =
        Selector::parse(&json.next_page_element.unwrap()).map_err(AppError::from_str)?;
    let post_link_selector =
        Selector::parse(&json.post_link_element).map_err(AppError::from_str)?;

    let mut url = params.url;
    let mut post_links: Vec<String> = Vec::new();
    while let (links, Some(next_url)) =
        get_post_links(url, &post_link_selector, &next_page_selector).await?
    {
        url = next_url;
        post_links.extend(links.into_iter().flatten());
    }

    Ok(Json(FeedContent {
        title: json.title,
        author: json.author,
        description: json.description,
        links: post_links,
    }))
}
