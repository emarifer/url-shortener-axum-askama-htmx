use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

/// Create a new `PgPoolOptions` instance and set the
/// maximum number of connections in the connection pool to 50.
pub async fn connect(db_url: &str) -> Result<Pool<Postgres>> {
    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(db_url)
        .await
        .context("Error: 🔥 unable to connect to database!")?;

    println!("🚀 Successfully connected to database!");

    Ok(db)
}
