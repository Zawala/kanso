//! The anonymous usage count: its one-time notice, the feature names
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

    /// Notes that the app was launched (once per process).
    pub fn launched(&self) -> Result<(), KansoError> {
        self.0.launched()
    }

    /// Notes that a foreground stretch of `secs` seconds ended.
    pub fn end_session(&self, secs: u64) -> Result<(), KansoError> {
        self.0.end_session(secs)
    }

    /// Sends what is due (the daily count, a launch, ended sessions and
    /// feature counts), if this is a release build and the notice was
    /// acknowledged. `locale` is the UI language code, `theme_mode` is
    /// `light`, `dark` or `system`, `theme` the theme ID in use; the last
    /// two may be empty to leave them out.
    pub fn send_due(
        &self,
        tablet: bool,
        os_version: String,
        ai: bool,
        locale: String,
        theme_mode: String,
        theme: String,
    ) -> Result<bool, KansoError> {
        self.0
            .send_due(tablet, &os_version, ai, &locale, &theme_mode, &theme)
    }
}
