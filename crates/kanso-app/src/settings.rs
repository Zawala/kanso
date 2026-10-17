//! The Settings window.
//!
//! Switches and menus apply at once. Addresses, ports, names and URLs are
//! changed in a prompt and applied only when they check out, so the API
//! doesn't restart and sync doesn't run on every keystroke.

use std::rc::Rc;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::setting::{
    NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings,
};
use gpui_kit::component::{ActiveTheme as _, Sizable as _, WindowExt as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::config::ThemePreference;
use crate::workspace::{AppState, apply_theme, for_each_workspace};
use crate::{dialogs, inkdrop, launch, paths, services};

fn config(cx: &App) -> &crate::config::AppConfig {
    &cx.global::<AppState>().config
}

/// A value with a Change… button that asks for a new one. `apply` returns
/// why a value can't be used, and the prompt stays open until it can.
fn changeable(
    id: &'static str,
    title: &'static str,
    current: impl Fn(&App) -> String + 'static,
    apply: impl Fn(String, &mut App) -> Result<(), String> + 'static,
) -> SettingField<SharedString> {
    let current = Rc::new(current);
    let apply = Rc::new(apply);
    SettingField::render(move |_, _, cx| {
        let value = current(cx);
        let current = current.clone();
        let apply = apply.clone();
        h_flex()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .when(value.is_empty(), |el| {
                        el.text_color(cx.theme().muted_foreground)
                    })
                    .child(if value.is_empty() {
                        "Not set".to_owned()
                    } else {
                        value
                    }),
            )
            .child(
                Button::new(id)
                    .small()
                    .label("Change…")
                    .on_click(move |_, window, cx| {
                        let apply = apply.clone();
                        dialogs::prompt_checked(
                            title,
                            &current(cx),
                            "Save",
                            window,
                            cx,
                            move |value, _, cx| apply(value, cx),
                        );
                    }),
            )
    })
}

/// The API port from what was typed.
pub fn parse_port(value: &str) -> Result<u16, String> {
    match value.trim().parse::<u16>() {
        Ok(port) if port >= 1024 => Ok(port),
        _ => Err("Use a port from 1024 to 65535.".into()),
    }
}

/// The API username from what was typed. Basic authentication can't carry
/// a colon in the username.
pub fn parse_username(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.contains(':') || value.chars().any(char::is_control) {
        Err("The username can't be empty or contain a colon.".into())
    } else {
        Ok(value.to_owned())
    }
}

pub struct SettingsWindow;

impl SettingsWindow {
    /// Opens Settings, or brings the open one forward.
    pub fn open(cx: &mut App) {
        for window in cx.windows() {
            let found = window
                .update(cx, |root, window, cx| {
                    let is_settings =
                        root.downcast::<gpui_kit::base::Root>()
                            .ok()
                            .is_some_and(|root| {
                                root.read(cx)
                                    .view()
                                    .clone()
                                    .downcast::<SettingsWindow>()
                                    .is_ok()
                            });
                    if is_settings {
                        window.activate_window();
                    }
                    is_settings
                })
                .unwrap_or(false);
            if found {
                return;
            }
        }
        let bounds = Bounds::centered(None, size(px(760.), px(560.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Kanso Settings".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| SettingsWindow)).ok();
    }
}

fn appearance_page() -> SettingPage {
    SettingPage::new("Appearance").group(
        SettingGroup::new()
            .item(SettingItem::new(
                "Theme",
                SettingField::dropdown(
                    vec![
                        ("system".into(), "Match System".into()),
                        ("light".into(), "Light".into()),
                        ("dark".into(), "Dark".into()),
                    ],
                    |cx| match config(cx).appearance.theme {
                        ThemePreference::System => "system".into(),
                        ThemePreference::Light => "light".into(),
                        ThemePreference::Dark => "dark".into(),
                    },
                    |value, cx| {
                        let theme = match value.as_ref() {
                            "light" => ThemePreference::Light,
                            "dark" => ThemePreference::Dark,
                            _ => ThemePreference::System,
                        };
                        AppState::update_config(cx, |config| config.appearance.theme = theme);
                        apply_theme(theme, None, cx);
                        cx.refresh_windows();
                    },
                ),
            ))
            .item(SettingItem::new(
                "Editor font size",
                SettingField::number_input(
                    NumberFieldOptions {
                        min: 10.0,
                        max: 28.0,
                        step: 1.0,
                    },
                    |cx| f64::from(config(cx).appearance.editor_font_size),
                    |value, cx| {
                        let size = value.clamp(10.0, 28.0) as f32;
                        AppState::update_config(cx, |config| {
                            config.appearance.editor_font_size = size
                        });
                        for_each_workspace(cx, |workspace, cx| {
                            workspace.editor.update(cx, |editor, cx| {
                                editor.font_size = size;
                                cx.notify();
                            });
                        });
                    },
                ),
            )),
    )
}

fn api_page() -> SettingPage {
    SettingPage::new("Local API").group(
        SettingGroup::new()
            .description(
                "Kanso serves Inkdrop's local HTTP API, so Inkdrop's MCP server (@inkdropapp/mcp-server) \
                 and other Inkdrop API clients work unchanged. Kanso's own MCP endpoint is /mcp on the same address. \
                 Quit Inkdrop first if it uses the same port.",
            )
            .item(SettingItem::new(
                "Enable the local API",
                SettingField::switch(
                    |cx| config(cx).server.enabled,
                    |on, cx| {
                        AppState::update_config(cx, |config| config.server.enabled = on);
                        services::apply_server_settings(cx);
                    },
                ),
            ))
            .item(
                SettingItem::new(
                    "Address",
                    changeable(
                        "change-address",
                        "API Address",
                        |cx| config(cx).server.bind_address.clone(),
                        |value, cx| {
                            let ip: std::net::IpAddr = value
                                .parse()
                                .map_err(|_| "Enter an IP address, such as 127.0.0.1.".to_owned())?;
                            AppState::update_config(cx, |config| {
                                config.server.bind_address = ip.to_string()
                            });
                            services::apply_server_settings(cx);
                            Ok(())
                        },
                    ),
                )
                .description("127.0.0.1 keeps the API on this Mac."),
            )
            .item(SettingItem::new(
                "Port",
                changeable(
                    "change-port",
                    "API Port",
                    |cx| config(cx).server.port.to_string(),
                    |value, cx| {
                        let port = parse_port(&value)?;
                        AppState::update_config(cx, |config| config.server.port = port);
                        services::apply_server_settings(cx);
                        Ok(())
                    },
                ),
            ))
            .item(SettingItem::new(
                "Username",
                changeable(
                    "change-username",
                    "API Username",
                    |cx| config(cx).server.username.clone(),
                    |value, cx| {
                        let username = parse_username(&value)?;
                        AppState::update_config(cx, |config| config.server.username = username);
                        services::apply_server_settings(cx);
                        Ok(())
                    },
                ),
            ))
            .item(SettingItem::new(
                "Password",
                SettingField::render(|_, _, cx| {
                    h_flex()
                        .gap_2()
                        .child(div().text_color(cx.theme().muted_foreground).child("••••••••••••"))
                        .child(Button::new("copy-password").small().label("Copy").on_click(|_, window, cx| {
                            let password = config(cx).server.password.clone();
                            cx.write_to_clipboard(ClipboardItem::new_string(password));
                            window.push_notification(Notification::info("Password copied."), cx);
                        }))
                        .child(Button::new("change-password").small().label("Change…").on_click(|_, window, cx| {
                            dialogs::prompt_text("New API Password", "", "Change", window, cx, |password, _, cx| {
                                AppState::update_config(cx, |config| config.server.password = password);
                                services::apply_server_settings(cx);
                            });
                        }))
                }),
            ))
            .item(
                SettingItem::new(
                    "Inkdrop's settings",
                    SettingField::render(|_, _, _| {
                        Button::new("copy-inkdrop")
                            .small()
                            .label("Copy from Inkdrop…")
                            .on_click(|_, window, cx| {
                                dialogs::confirm(
                                    "Use Inkdrop's port, username and password?",
                                    "Inkdrop keeps its password encrypted with a key in your Keychain. macOS will ask you to allow Kanso to read “inkdrop Safe Storage”.",
                                    "Continue",
                                    window,
                                    cx,
                                    |window, cx| {
                                        inkdrop::start_copy_credentials(window, cx, |_, cx| services::apply_server_settings(cx));
                                    },
                                );
                            })
                    }),
                )
                .description("Lets clients set up for Inkdrop keep their URL and credentials."),
            )
            .item(SettingItem::new(
                "Status",
                SettingField::render(|_, _, cx| div().text_sm().child(cx.global::<services::Services>().api_status())),
            )),
    )
}

fn sync_page() -> SettingPage {
    SettingPage::new("Sync").group(
        SettingGroup::new()
            .description(
                "Sync notes between Macs through your own Cloudflare Worker (see cloud/README.md).",
            )
            .item(SettingItem::new(
                "Sync automatically",
                SettingField::switch(
                    |cx| config(cx).sync.enabled,
                    |on, cx| {
                        AppState::update_config(cx, |config| config.sync.enabled = on);
                        services::apply_sync_settings(cx);
                    },
                ),
            ))
            .item(SettingItem::new(
                "Worker URL",
                changeable(
                    "change-endpoint",
                    "Worker URL",
                    |cx| config(cx).sync.endpoint.clone().unwrap_or_default(),
                    |value, cx| {
                        let endpoint = kanso_sync::check_endpoint(&value).map_err(|err| {
                            let mut message = err.to_string();
                            message[..1].make_ascii_uppercase();
                            format!("{message}.")
                        })?;
                        AppState::update_config(cx, |config| config.sync.endpoint = Some(endpoint));
                        services::apply_sync_settings(cx);
                        Ok(())
                    },
                ),
            ))
            .item(SettingItem::new(
                "Sync every",
                // A menu rather than a number field: each change restarts
                // sync, which shouldn't happen per keystroke.
                SettingField::dropdown(
                    [1u64, 5, 15, 30, 60, 240]
                        .into_iter()
                        .map(|minutes| {
                            let label = match minutes {
                                1 => "Minute".to_owned(),
                                60 => "Hour".to_owned(),
                                m if m % 60 == 0 => format!("{} hours", m / 60),
                                m => format!("{m} minutes"),
                            };
                            (minutes.to_string().into(), label.into())
                        })
                        .collect(),
                    |cx| (config(cx).sync.interval_secs / 60).to_string().into(),
                    |value, cx| {
                        let minutes: u64 = value.parse().unwrap_or(15).clamp(1, 1440);
                        AppState::update_config(cx, |config| {
                            config.sync.interval_secs = minutes * 60
                        });
                        services::apply_sync_settings(cx);
                    },
                ),
            ))
            .item(
                SettingItem::new(
                    "Access token",
                    SettingField::render(|_, _, _| {
                        h_flex()
                            .gap_2()
                            .child(Button::new("set-token").small().label("Set…").on_click(
                                |_, window, cx| {
                                    dialogs::prompt_text(
                                        "Sync Access Token",
                                        "",
                                        "Save",
                                        window,
                                        cx,
                                        |token, window, cx| match services::save_sync_token(&token)
                                        {
                                            Ok(()) => {
                                                window.push_notification(
                                                    Notification::success("Saved to the Keychain."),
                                                    cx,
                                                );
                                                services::apply_sync_settings(cx);
                                            }
                                            Err(err) => window.push_notification(
                                                Notification::error(format!("{err:#}")),
                                                cx,
                                            ),
                                        },
                                    );
                                },
                            ))
                            .child(
                                Button::new("sync-now")
                                    .small()
                                    .primary()
                                    .label("Sync Now")
                                    .on_click(|_, _, cx| {
                                        services::sync_now(cx);
                                    }),
                            )
                    }),
                )
                .description("Kept in your Keychain, never in Kanso's settings file."),
            )
            .item(SettingItem::new(
                "Status",
                SettingField::render(|_, _, cx| {
                    div()
                        .text_sm()
                        .child(cx.global::<services::Services>().sync_status())
                }),
            )),
    )
}

fn general_page() -> SettingPage {
    SettingPage::new("General").group(
        SettingGroup::new()
            .item(SettingItem::new(
                "Open Kanso at login",
                SettingField::switch(
                    |cx| config(cx).launch_at_login,
                    |on, cx| match launch::set_enabled(
                        &paths::launch_agent_path(),
                        on,
                        &cx.global::<AppState>().data_dir,
                    ) {
                        Ok(()) => AppState::update_config(cx, |config| config.launch_at_login = on),
                        Err(err) => eprintln!("kanso: launch at login: {err:#}"),
                    },
                ),
            ))
            .item(
                SettingItem::new(
                    "Data folder",
                    SettingField::render(|_, _, cx| {
                        let dir = cx.global::<AppState>().data_dir.clone();
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(dir.display().to_string()),
                            )
                            .child(
                                Button::new("reveal-data")
                                    .small()
                                    .ghost()
                                    .label("Show in Finder")
                                    .on_click(move |_, _, cx| {
                                        cx.reveal_path(&dir);
                                    }),
                            )
                    }),
                )
                .description("Kanso backs up its database here once a day and keeps the last 14."),
            )
            .item(SettingItem::new(
                "Back up now",
                SettingField::render(|_, _, _| {
                    Button::new("backup-now")
                        .small()
                        .label("Back Up")
                        .on_click(|_, window, cx| {
                            let state = cx.global::<AppState>();
                            let store = state.store.clone();
                            let dir = paths::backups_dir(&state.data_dir);
                            let path = dir.join(format!("kanso-{}.db", kanso_core::now_ms()));
                            // Copying a large library takes a while; keep the
                            // window responsive.
                            let backup = cx.background_spawn({
                                let path = path.clone();
                                async move {
                                    std::fs::create_dir_all(&dir)?;
                                    store.backup_to(&path)?;
                                    anyhow::Ok(())
                                }
                            });
                            window
                                .spawn(cx, async move |cx| {
                                    let result = backup.await;
                                    cx.update(|window, cx| {
                                        let notification = match result {
                                            Ok(()) => Notification::success(format!(
                                                "Backed up to {}",
                                                path.display()
                                            )),
                                            Err(err) => Notification::error(format!(
                                                "Backup failed: {err:#}"
                                            )),
                                        };
                                        window.push_notification(notification, cx);
                                    })
                                    .ok();
                                })
                                .detach();
                        })
                }),
            ))
            .item(SettingItem::new(
                "Inkdrop",
                SettingField::render(|_, _, _| {
                    Button::new("import-inkdrop")
                        .small()
                        .label("Import from Inkdrop…")
                        .on_click(|_, window, cx| {
                            inkdrop::start_import(window, cx);
                        })
                }),
            )),
    )
}

impl Render for SettingsWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(Settings::new("kanso-settings").small().pages([
                general_page(),
                appearance_page(),
                api_page(),
                sync_page(),
            ]))
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_port, parse_username};
    use proptest::prelude::*;

    #[test]
    fn examples() {
        assert_eq!(parse_port(" 19840 "), Ok(19840));
        assert!(parse_port("1023").is_err());
        assert!(parse_port("1984o").is_err());
        assert!(parse_port("65536").is_err());
        assert_eq!(parse_username(" bob ").as_deref(), Ok("bob"));
        assert!(parse_username("bob:x").is_err());
        assert!(parse_username("  ").is_err());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Round-trip: every usable port is accepted as typed, and nothing
        /// outside 1024..=65535 is.
        #[test]
        fn prop_ports_round_trip(port in any::<u16>()) {
            let parsed = parse_port(&port.to_string());
            prop_assert_eq!(parsed.is_ok(), port >= 1024);
            if let Ok(parsed) = parsed {
                prop_assert_eq!(parsed, port);
            }
        }

        /// Refinement: an accepted username is trimmed, non-empty and
        /// usable in Basic authentication.
        #[test]
        fn prop_usernames_fit_basic_auth(value in "\\PC{0,12}|[a-z:]{0,6}") {
            if let Ok(name) = parse_username(&value) {
                prop_assert!(!name.is_empty() && !name.contains(':'));
                prop_assert_eq!(name.trim(), name.as_str());
            }
        }
    }
}
