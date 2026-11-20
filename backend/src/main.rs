mod api;
mod config;
mod db;
mod model;
mod validate;

use std::{net::SocketAddr, sync::Arc};

use api::AppState;
use config::Config;
use db::Db;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::from_env()?);
    let db = Db::from_config(&config);

    if std::env::args().any(|arg| arg == "--backup") {
        let records = db.list().await?;
        println!("{}", serde_json::to_string_pretty(&records)?);
        return Ok(());
    }

    let app = api::router(AppState::new(config.clone(), db));
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
