mod room;

use crate::room::Room;

use std::error::Error;
use tokio::net::TcpListener;

use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

fn setup_logging() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logging();
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    let mut default_room = Room::new(listener);
    info!("Listening on: {addr}");
    default_room.run().await?;
    Ok(())
}
