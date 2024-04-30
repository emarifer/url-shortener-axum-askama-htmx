use anyhow::Result;
use dotenvy::dotenv;
use dotenvy_macro::dotenv;

mod db;
mod http;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from the `.env` file
    dotenv().ok();

    // Retrieve the value of the `DATABASE_URL` env variable
    let db_url = dotenv!("DATABASE_URL");

    // Connect to Postgres database
    let db = db::connect(db_url).await?;

    // Start the http server
    http::serve(db).await?;

    Ok(())
}

/* HOT RELOADING COMMAND (avoiding the `.pgdata` folder):
cargo watch -x run -w src -w assets -w templates
*/

/* REFERENCES:
https://freedium.cfd/https://deid84.medium.com/crafting-a-rust-url-shortener-part-1-setting-up-your-project-9babe97b8962
https://freedium.cfd/https://levelup.gitconnected.com/crafting-a-rust-url-shortener-part-2-creating-apis-1753b4789816
https://freedium.cfd/https://levelup.gitconnected.com/crafting-a-rust-url-shortener-part-3-building-frontend-33409282083f
https://github.com/deid84/shorturl-rs
https://joeymckenzie.tech/blog/templates-with-rust-axum-htmx-askama

https://docs.rs/axum/latest/axum/
https://docs.rs/askama/latest/askama/
https://djc.github.io/askama/

SIMPLIFY RUST ERROR HANDLING WITH ANYHOW. SEE:
https://freedium.cfd/https://deid84.medium.com/simplify-rust-error-handling-with-anyhow-f680410e70f9

If you would like to create reversible migrations with corresponding
"up" and "down" scripts, you use the -r flag when creating the first migration:
https://docs.rs/crate/sqlx-cli/latest
*/
