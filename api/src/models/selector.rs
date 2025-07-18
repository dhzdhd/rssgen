use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::feed::Feed;

#[derive(Queryable, Selectable, Serialize, Deserialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::post_selectors)]
#[diesel(belongs_to(Feed, foreign_key = feed_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostSelector {
    pub id: i32,
    pub feed_id: i32,
    pub post_list_element: String,
    pub post_title_element: String,
    pub post_content_element: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::post_selectors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostContentSelector {
    pub post_title_element: String,
    pub post_content_element: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::post_selectors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostListSelector {
    pub post_list_element: String,
}
