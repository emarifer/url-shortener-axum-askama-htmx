use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Host, Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use url::Url;

use super::{
    service::{convert_datetime, resolve_short_url, shorten_url},
    AppState,
};

// Change it with the URL where you'll host your project
// const BASE_URL: &str = "localhost:8086/";

// Struct for holding form data
#[derive(Deserialize)]
pub struct FormData {
    url: String,
}

/// Handler to respond to `GET` requests from the `URL`.
pub async fn get_url(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(url): Path<String>,
) -> impl IntoResponse {
    // Attempt to resolve the short URL to its corresponding long URL
    match resolve_short_url(&state.read().await.db, url).await {
        Ok(result) => Redirect::to(&result).into_response(),
        Err(_err) => Redirect::to("/404").into_response(),
    }
}

/// Handler to respond to `POST` requests and generate,
/// if possible, a short `URL`.
pub async fn post_url(
    headers: HeaderMap,
    Host(hostname): Host, // See note below
    State(state): State<Arc<RwLock<AppState>>>,
    Form(form_data): Form<FormData>,
) -> impl IntoResponse {
    // println!("{hostname}");
    // println!("{:?}", headers["x-timezone"]);
    // Check if the provided URL is valid
    if Url::parse(&form_data.url).is_ok() {
        // Generate url random part
        match shorten_url(&state.write().await.db, form_data.url).await {
            Ok(result) => HtmlTemplate(ResultTemplate {
                url: format!("{}/{}", hostname, result.id),
                datetime: convert_datetime(
                    headers["x-timezone"].to_str().unwrap(),
                    result.created_at,
                ),
            }),
            Err(err) => HtmlTemplate(ResultTemplate {
                url: format!("Something went wrong: {}", err),
                datetime: String::default(),
            }),
        }
    } else {
        HtmlTemplate(ResultTemplate {
            url: "This is not a valid URL".to_string(),
            datetime: String::default(),
        })
    }
}

/// Handler 404 Error.
pub async fn handler_404() -> impl IntoResponse {
    HtmlTemplate(ErrTemplate {
        title: "Error 404".to_string(),
    })
}

#[derive(Template)]
#[template(path = "404.html")]
struct ErrTemplate {
    title: String,
}

/// Handler to serve the application template.
pub async fn app() -> impl IntoResponse {
    HtmlTemplate(AppTemplate {
        title: "A Rust URL Shortener App".to_string(),
    })
}

#[derive(Template)]
#[template(path = "app.html")]
struct AppTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "partials/result.html")]
struct ResultTemplate {
    url: String,
    datetime: String,
}

/* ***** Template Rendering ***** */

/// A wrapper type that we'll use to encapsulate HTML parsed
/// by askama into valid HTML for axum to serve.
struct HtmlTemplate<T>(T);

/// Allows us to convert Askama HTML templates into valid HTML
/// for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

/* IMPORTANT!! IN AXUM, THE LAST ARGUMENT CANNOT IMPLEMENT `FromRequestParts`. SEE:
https://docs.rs/axum/latest/axum/handler/trait.Handler.html#debugging-handler-type-errors
https://docs.rs/axum/latest/axum/extract/trait.FromRequestParts.html

https://stackoverflow.com/questions/76307624/unexplained-trait-bound-no-longer-satisfied-when-modifying-axum-handler-body
https://github.com/emarifer/axum-postgres-api/blob/main/src/handler.rs#L32-L71
*/
