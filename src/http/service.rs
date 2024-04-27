use anyhow::Result;
use chrono::{offset::TimeZone, Local, NaiveDateTime};
use chrono_tz::Tz;
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{Error, Pool, Postgres};

use super::model::Url;

const MAX_RETRIES: usize = 5;
const SHORTURL_LEN: usize = 4;

/// Verifies if a short URL exists in our database.
/// If it finds a corresponding entry, it returns the associated full URL,
/// which will be utilized in the subsequent handler function
/// defined later in the code.
pub async fn resolve_short_url(db: &Pool<Postgres>, url: String) -> Result<String, Error> {
    // Check if random string already exists in db
    match sqlx::query_as!(Url, r#"SELECT * FROM url WHERE id = $1;"#, url)
        .fetch_one(db)
        .await
    {
        Ok(data) => {
            // The URL has been found in the database,
            // returning the corresponding long URL
            Ok(data.long_url)
        }
        Err(err) => {
            // An error occurred during the database operation,
            // returning the error
            Err(err)
        }
    }
}

/// Receives the full URL that needs to be shortened.
/// Upon receiving this input, it utilizes the rand crate
/// to generate a random 4-character base62 string
/// (you can adjust the length as needed).
pub async fn shorten_url(db: &Pool<Postgres>, url: String) -> Result<Url, Error> {
    // Loop for a maximum number of retries
    for _ in 0..MAX_RETRIES {
        let rng = rand::thread_rng();

        // Generate a random string for the short URL
        let random_string = rng
            .sample_iter(&Alphanumeric)
            .take(SHORTURL_LEN)
            .map(char::from)
            .collect::<String>();

        // Try to insert data in the db
        match sqlx::query_as!(
            Url,
            r#"INSERT INTO url (id, long_url)
            VALUES ($1, $2)
            RETURNING *;"#,
            random_string,
            url
        )
        .fetch_one(db)
        .await
        {
            Ok(url) => {
                // Successful insertion, return the object
                // containing the generated random string
                return Ok(url);
            }
            Err(err) => {
                // Check if the error is due to a unique constraint
                // violation (i.e, value already exists)
                if let sqlx::Error::Database(db_err) = &err {
                    if db_err.is_foreign_key_violation() {
                        // Retry inserting with a new random string
                        continue;
                    }
                    // If it's not a unique constraint violation or
                    // if the maximum retries are reached, return the error
                    return Err(err);
                }
            }
        }
    }

    // If maximum retries are reached without successful insertion,
    // return an error
    Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Maximum retries reached without successful insertion",
    )))
}

/// convert_datetime converts the datetime format from the
/// database (UTC timestamp) to a string in RFC822Z format,
/// taking the client's timezone (&str) and a datetime (NaiveDateTime).
pub fn convert_datetime(tzone: &str, dt: NaiveDateTime) -> String {
    let tz = tzone.parse::<Tz>().unwrap();
    let converted = Local.from_utc_datetime(&dt);
    let dttz = converted.with_timezone(&tz).to_rfc2822();

    // conversion to RFC822Z format
    let chars = dttz.chars().collect::<Vec<_>>();
    let first_part = chars[5..22].iter().collect::<String>();
    let last_part = chars[25..].iter().collect::<String>();

    format!("{}{}", first_part, last_part)
}
