use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// This struct represents the URL db table: `id` and `long_url`.
///
#[derive(Deserialize, Serialize)]
pub struct Url {
    /// The random shorturl string.
    pub id: String,
    /// The long URL corresponding to the short URL.
    pub long_url: String,
    /// Date & time of insertion of the item in the DB.
    pub created_at: NaiveDateTime,
}
