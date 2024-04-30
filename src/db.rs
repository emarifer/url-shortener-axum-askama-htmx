use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

const SET_TIMEOUT_DB_CONN: u64 = 5;

/// Create a new `PgPoolOptions` instance and set the
/// maximum number of connections in the connection pool to 50.
pub async fn connect(db_url: &str) -> Result<Pool<Postgres>> {
    let db = PgPoolOptions::new()
        // See note below ↓↓
        .acquire_timeout(Duration::from_secs(SET_TIMEOUT_DB_CONN))
        .max_connections(50)
        .connect(db_url)
        .await
        .context("Error: 🔥 unable to connect to database!")?;

    println!("🚀 Successfully connected to database!");

    Ok(db)
}

/* LIMIT DATABASE CONNECTION WAITING TIME. SEE:
https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.acquire_timeout
*/
