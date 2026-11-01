//! The anonymous daily usage count: its one-time notice, the feature names
//! the product records, and the batch sent while the app is open. The rules
//! and the ledger live in `kanso-mobile` / `kanso-metrics`; this is only the
//! thin bridge to Dart.

use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, MobileMetrics};

#[frb(opaque)]
pub struct KansoMetrics(MobileMetrics);

impl KansoMetrics {
    pub(super) fn new(metrics: MobileMetrics) -> KansoMetrics {
        KansoMetrics(metrics)
    }

    pub fn notice_seen(&self) -> Result<bool, KansoError> {
        self.0.notice_seen()
    }

    pub fn mark_notice_seen(&self) -> Result<(), KansoError> {
        self.0.mark_notice_seen()
    }

    /// Counts a feature by its closed name. The Rust side rejects unknown
    /// names; Dart never gets an open-ended event API.
    pub fn record(&self, feature: String) -> Result<(), KansoError> {
        self.0.record(&feature)
    }

    /// Sends today's count and any waiting feature events, if this is a
    /// release build and the notice was acknowledged.
    pub fn send_due(&self, tablet: bool, os_version: String, ai: bool) -> Result<bool, KansoError> {
        self.0.send_due(tablet, &os_version, ai)
    }
}
