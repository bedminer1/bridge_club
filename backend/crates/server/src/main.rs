mod auth;
mod db;
mod routes;
mod session;

use std::net::SocketAddr;

use axum::http::{header, HeaderName, HeaderValue};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

#[tokio::main]
async fn main() {
    // Load .env file (if present)
    dotenvy::dotenv().ok();

    // Initialize tracing (logging)
    tracing_subscriber::fmt::init();

    // Initialize the database pool
    let db_pool = db::DbPool::from_env()
        .await
        .expect("Failed to connect to database");

    // Run schema migrations
    db::run_migrations(&db_pool)
        .await
        .expect("Failed to run database migrations");

    // Create shared application state
    let state = session::new_app_state(db_pool).await;

    // Build the router
    let app = routes::routes(state);

    // CORS: allow the SvelteKit dev server with credentials
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(
            HeaderValue::from_static("http://localhost:5173"),
        ))
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static("x-session-token"),
        ]))
        .allow_credentials(true);

    let app = app.layer(cors);

    // Bind and serve
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to 127.0.0.1:3000");

    tracing::info!("Bridge Club server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
