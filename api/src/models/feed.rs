use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::feeds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Feed {
    pub id: i32,
    pub title: String,
    pub link: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::feeds)]
pub struct NewFeed {
    pub title: String,
    pub link: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct FeedContent {
    pub title: String,
    pub content: String,
}
