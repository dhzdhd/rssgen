use scraper::{Html, Selector};

use crate::{error::AppError, models::selector::PostContentSelector};

pub async fn get_post_html(
    html: &str,
    selectors: &PostContentSelector,
) -> Result<(String, String), AppError> {
    let parsed = Html::parse_document(&html);
    let title_selector =
        Selector::parse(&selectors.post_title_element).map_err(AppError::from_str)?;
    let content_selector =
        Selector::parse(&selectors.post_content_element).map_err(AppError::from_str)?;

    let title_html = parsed.select(&title_selector).next().unwrap().inner_html();
    let content_html = parsed
        .select(&content_selector)
        .next()
        .unwrap()
        .inner_html();

    Ok((title_html, content_html))
}

pub async fn get_feed_html() -> Result<String, AppError> {
    unimplemented!()
}
