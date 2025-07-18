// @generated automatically by Diesel CLI.

diesel::table! {
    feeds (id) {
        id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        description -> Nullable<Text>,
        #[max_length = 255]
        author -> Varchar,
        link -> Text,
        pages -> Array<Nullable<Text>>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    post_selectors (id) {
        id -> Int4,
        feed_id -> Int4,
        post_list_element -> Text,
        post_title_element -> Text,
        post_content_element -> Text,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        feed_id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        link -> Text,
        content -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(post_selectors -> feeds (feed_id));
diesel::joinable!(posts -> feeds (feed_id));

diesel::allow_tables_to_appear_in_same_query!(
    feeds,
    post_selectors,
    posts,
);
