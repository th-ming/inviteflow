mod api;
mod config;
mod db;
mod model;
mod validate;

use std::{net::SocketAddr, sync::Arc};

use api::AppState;
use config::Config;
use db::{Db, GithubDb};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(index) = args.iter().position(|arg| arg == "--hash-password") {
        let password = args
            .get(index + 1)
            .ok_or("Thiếu mật khẩu sau --hash-password")?;
        println!("{}", api::hash_password(password)?);
        return Ok(());
    }

    let config = Arc::new(Config::from_env()?);
    let db = Db::from_config(&config);
    let admin_db = GithubDb::new(
        &config.github_repo,
        &config.admin_file,
        &config.github_token,
    );
    let content_db = GithubDb::new(&config.github_repo, "content.json", &config.github_token);

    if args.iter().any(|arg| arg == "--backup") {
        let records = db.list().await?;
        println!("{}", serde_json::to_string_pretty(&records)?);
        return Ok(());
    }

    let app = api::router(AppState::new(config.clone(), db, admin_db, content_db));
    let address: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(address).await?;
    println!("api listening on http://{address}");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
