//! Sideros website server
//!
//! This serves the main Sideros website with documentation,
//! installation instructions, and wiki content.

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

// Template definitions
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "install.html")]
struct InstallTemplate;

#[derive(Template)]
#[template(path = "wiki.html")]
struct WikiTemplate;

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate;

// Handler functions
async fn index() -> impl IntoResponse {
    match IndexTemplate.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {}", err),
        )
            .into_response(),
    }
}

async fn install() -> impl IntoResponse {
    match InstallTemplate.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {}", err),
        )
            .into_response(),
    }
}

async fn wiki() -> impl IntoResponse {
    match WikiTemplate.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {}", err),
        )
            .into_response(),
    }
}

async fn about() -> impl IntoResponse {
    match AboutTemplate.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {}", err),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    // Build our application with routes
    let app = Router::new()
        // Page routes
        .route("/", get(index))
        .route("/install", get(install))
        .route("/wiki", get(wiki))
        .route("/about", get(about))
        // Static file serving
        .nest_service("/static", ServeDir::new("static"));

    // Run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Sideros website running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
