mod db;
mod routes;
mod session;

use std::net::SocketAddr;

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
