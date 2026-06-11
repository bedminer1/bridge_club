use libsql::Builder;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let url = env::var("DATABASE_URL").unwrap_or_else(|_| "file:local.db".into());
    let auth_token = env::var("DATABASE_AUTH_TOKEN").ok();
    let db = if url.starts_with("libsql://") || url.starts_with("https://") {
        Builder::new_remote(url, auth_token.unwrap()).build().await?
    } else {
        Builder::new_local(url).build().await?
    };
    let conn = db.connect()?;

    let mut rows = conn.query("SELECT id, username FROM users ORDER BY id", libsql::params![]).await?;
    while let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        println!("{}: {}", id, name);
    }
    Ok(())
}
