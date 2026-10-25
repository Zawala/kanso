//! Kanso's plugins, with notes kept safe.
//!
//! A plugin is JavaScript in its own `kanso-plugin-host` process with only
//! the host calls its manifest asks for and the person granted. Everything
//! that decides what a plugin may do lives here, apart from the app's
//! windows: the manifest and its caps ([`manifest`]), the grant
//! ([`grant`]), the scoped reads of the store ([`scope`]), the router that
//! checks and answers every host call ([`router`]), the staged writes and
//! their review ([`batch`]), the supervisor that starts, deadlines and kills
//! each host ([`supervisor`]), installing and removing ([`install`]), the
//! network proxy ([`net`]), panels ([`panel`]) and the signed directory
//! ([`registry`], [`bundle`]).
//!
//! The one rule: a plugin has no ambient authority. Each capability is a
//! function here; with no function, there is no path.

pub mod batch;
pub mod bundle;
pub mod consent;
pub mod grant;
pub mod install;
pub mod manifest;
pub mod model;
pub mod net;
pub mod panel;
pub mod panel_model;
pub mod registry;
pub mod registry_model;
pub mod router;
pub mod scope;
pub mod state;
pub mod supervisor;

pub use kanso_plugin_host::protocol;
