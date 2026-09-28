use std::collections::HashMap;

use serde::Serialize;
use tokio::sync::broadcast::Sender;
use tracing::{debug, error, trace};
use zbus::interface;
use zvariant::OwnedValue;

const SUPPORTED_CAPABILITY: &[&str] = &["body", "body-hyperlinks", "persistence"];

#[derive(Debug)]
pub struct Notifications {
    last_id: u32,
    notif_tx: Sender<Notif>,
}

impl Notifications {
    pub const fn new(notif_tx: Sender<Notif>) -> Self {
        Self {
            last_id: 0,
            notif_tx,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Notif {
    pub id: u32,
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub hints: HashMap<String, OwnedValue>,
    pub expire_timeout: i32,
}

#[allow(clippy::unused_self)]
#[allow(clippy::too_many_arguments)]
#[allow(unused_variables)]
#[interface(name = "org.freedesktop.Notifications")]
impl Notifications {
    fn get_capabilities(&self) -> &[&str] {
        trace!("Ask GetCapabilities");
        SUPPORTED_CAPABILITY
    }

    #[zbus(out_args("id"))]
    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        debug!("Ask Notify");

        self.last_id = self.last_id.wrapping_add(1);
        let notif = Notif {
            id: self.last_id,
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
        };

        if let Err(err) = self.notif_tx.send(notif) {
            error!(?err);
        }

        self.last_id
    }

    fn close_notification(&self, id: u32) {
        trace!("Ask CloseNotification");
    }

    #[allow(clippy::unused_self)]
    #[zbus(out_args("name", "vendor", "version", "spec_version"))]
    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        trace!("Ask GetServerInformation");

        (
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_VERSION"),
            "1.3",
        )
    }
}
