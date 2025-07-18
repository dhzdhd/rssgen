use crate::{error::AppError, models::post::Post};

pub trait RSSGenerator {
    fn generate(post: Post) -> Result<String, AppError>;
}

pub struct AtomRSS;
pub struct RSS2;

impl RSSGenerator for AtomRSS {
    fn generate(post: Post) -> Result<String, AppError> {
        unimplemented!()
    }
}

impl RSSGenerator for RSS2 {
    fn generate(post: Post) -> Result<String, AppError> {
        unimplemented!()
    }
}
