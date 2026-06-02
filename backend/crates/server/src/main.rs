use std::net::SocketAddr;

use axum::{
    http::{header, HeaderName},
    routing::get,
};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

#[tokio::main]
async fn main() {
    // Load .env file (if present)
    dotenvy::dotenv().ok();

    // Initialize tracing (logging)
    tracing_subscriber::fmt::init();

    // Initialize the database pool
    let db_pool = bridge_server::db::DbPool::from_env()
        .await
        .expect("Failed to connect to database");

    // Run schema migrations
    bridge_server::db::run_migrations(&db_pool)
        .await
        .expect("Failed to run database migrations");

    // Create shared application state
    let state = bridge_server::session::new_app_state(db_pool).await;

    // Build the router — HTTP + WS routes
    let app = bridge_server::routes::routes(state.clone())
        .merge(
            axum::Router::new()
                .route("/ws", get(bridge_server::ws::ws_handler))
                .with_state(state),
        );

    // CORS: allow any origin (Vercel preview URLs vary)
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static("x-session-token"),
        ]));

    let app = app.layer(cors);

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to 0.0.0.0:3000");

    tracing::info!("Bridge Club server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
