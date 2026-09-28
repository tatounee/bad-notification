#![warn(clippy::pedantic)]

use color_eyre::Result;
use hyper_but_worse::{
    app,
    services::{Router, StaticFile, sse::ServerSendEvent},
};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use zbus::connection;

mod notification;
mod web_service;

use notification::Notifications;

use crate::web_service::NotifStream;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(EnvFilter::from_default_env())
        .init();

    // Prepare dbus and web service
    let (notif_stream, notif_tx) = NotifStream::new();
    let dbus_service = Notifications::new(notif_tx);
    let web_service = ServerSendEvent::new(notif_stream);

    // Initialise dbus server
    let dbus_server = connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", dbus_service)?
        .build();

    // Initialise web server
    let router = Router::new()
        .route("/notifs", web_service)
        .route("/", StaticFile::new("./static")?);

    let conn = TcpListener::bind(("::1", 8800)).await?;

    // Launch servers
    let _dbus = dbus_server.await?;
    app::run(conn, router).await?;

    Ok(())
}
