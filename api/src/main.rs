mod error;
mod models;
mod routes;
mod schema;

use crate::routes::feed::{analyze_url, create_feed, get_feeds};
use anyhow::{Ok, anyhow};
use axum::{Router, routing::get};
use deadpool_diesel::{
    Runtime,
    postgres::{Manager, Pool},
};
use diesel::{Connection, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenvy::dotenv;
use lambda_http::{run, tracing};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

async fn index() -> &'static str {
    "Hello World"
}

async fn healthcheck() -> &'static str {
    "Healthy"
}

async fn run_migrations(db_url: &str) -> Result<(), anyhow::Error> {
    let mut conn = PgConnection::establish(db_url)?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|err| anyhow!(err))?;

    println!("Ran migrations successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing::init_default_subscriber();

    if cfg!(debug_assertions) {
        dotenv()?;
    }

    let db_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL environment variable");

    run_migrations(&db_url).await?;

    let config = Manager::new(db_url, Runtime::Tokio1);
    let pool = Pool::builder(config).build().unwrap();

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(healthcheck))
        .route("/feeds", get(get_feeds).post(create_feed))
        .route("/feeds/analyze", get(analyze_url))
        .with_state(pool);

    if cfg!(debug_assertions) {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
        println!("Listening on http://localhost:8080");
        axum::serve(listener, app).await?;
    } else {
        run(app).await.map_err(|err| anyhow::anyhow!(err))?;
    }

    Ok(())
}
