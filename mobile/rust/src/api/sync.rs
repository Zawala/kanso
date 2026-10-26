//! Signing in, sync and the status the sidebar shows.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, ServerInfo, SyncHandle, SyncReportRow, SyncStatus};

#[frb(opaque)]
pub struct KansoSync(SyncHandle);

impl KansoSync {
    pub(super) fn new(handle: SyncHandle) -> KansoSync {
        KansoSync(handle)
    }

    pub fn default_server() -> String {
        SyncHandle::default_server()
    }

    pub fn status(&self) -> Result<SyncStatus, KansoError> {
        self.0.status()
    }

    /// The status now and each time it changes, until Dart stops listening.
    pub fn status_stream(&self, sink: StreamSink<SyncStatus>) {
        self.0.status_stream(move |status| sink.add(status).is_ok());
    }

    pub fn check_server(&self, url: String) -> Result<ServerInfo, KansoError> {
        self.0.check_server(url)
    }

    pub fn sign_in_password(
        &self,
        server: String,
        username: String,
        password: String,
        device_name: String,
    ) -> Result<SyncStatus, KansoError> {
        self.0
            .sign_in_password(server, username, password, device_name)
    }

    pub fn sign_out(&self) -> Result<(), KansoError> {
        self.0.sign_out()
    }

    /// The app is in front (`true`) or not.
    pub fn set_foreground(&self, foreground: bool) {
        self.0.set_foreground(foreground);
    }

    pub fn sync_now(&self) -> Result<SyncReportRow, KansoError> {
        self.0.sync_now()
    }
}
