use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::feed::Feed;

#[derive(Queryable, Selectable, Serialize, Deserialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(belongs_to(Feed, foreign_key = feed_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Post {
    pub id: i32,
    pub feed_id: i32,
    pub title: String,
    pub content: String,
    pub link: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::posts)]
pub struct NewPost {
    pub feed_id: i32,
    pub title: String,
    pub content: String,
    pub link: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostSelector {
    pub title: String,
    pub content: String,
}
