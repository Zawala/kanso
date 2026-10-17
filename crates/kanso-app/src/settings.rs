//! The Settings window.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::setting::{
    NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings,
};
use gpui_kit::component::{ActiveTheme as _, Sizable as _, WindowExt as _, h_flex};
use gpui_kit::*;

use crate::config::ThemePreference;
use crate::workspace::{AppState, apply_theme, for_each_workspace};
use crate::{dialogs, inkdrop, launch, paths, services};

fn config(cx: &App) -> &crate::config::AppConfig {
    &cx.global::<AppState>().config
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
                    SettingField::input(
                        |cx| config(cx).server.bind_address.clone().into(),
                        |value, cx| {
                            let value = value.trim().to_owned();
                            if value.parse::<std::net::IpAddr>().is_ok() {
                                AppState::update_config(cx, |config| config.server.bind_address = value);
                                services::apply_server_settings(cx);
                            }
                        },
                    ),
                )
                .description("127.0.0.1 keeps the API on this Mac."),
            )
            .item(SettingItem::new(
                "Port",
                SettingField::number_input(
                    NumberFieldOptions {
                        min: 1024.0,
                        max: 65535.0,
                        step: 1.0,
                    },
                    |cx| f64::from(config(cx).server.port),
                    |value, cx| {
                        let port = value.clamp(1024.0, 65535.0) as u16;
                        AppState::update_config(cx, |config| config.server.port = port);
                        services::apply_server_settings(cx);
                    },
                ),
            ))
            .item(SettingItem::new(
                "Username",
                SettingField::input(
                    |cx| config(cx).server.username.clone().into(),
                    |value, cx| {
                        let value = value.trim().to_owned();
                        if !value.is_empty() {
                            AppState::update_config(cx, |config| config.server.username = value);
                            services::apply_server_settings(cx);
                        }
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
                                    "Inkdrop keeps its password encrypted with a key in your Keychain. macOS will ask you to allow Kanso to read “Inkdrop Safe Storage”.",
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
                SettingField::input(
                    |cx| config(cx).sync.endpoint.clone().unwrap_or_default().into(),
                    |value, cx| {
                        let value = value.trim().trim_end_matches('/').to_owned();
                        if value.is_empty()
                            || value.starts_with("https://")
                            || value.starts_with("http://127.0.0.1")
                        {
                            AppState::update_config(cx, |config| {
                                config.sync.endpoint = (!value.is_empty()).then_some(value);
                            });
                            services::apply_sync_settings(cx);
                        }
                    },
                ),
            ))
            .item(SettingItem::new(
                "Every (minutes)",
                SettingField::number_input(
                    NumberFieldOptions {
                        min: 1.0,
                        max: 1440.0,
                        step: 1.0,
                    },
                    |cx| (config(cx).sync.interval_secs / 60) as f64,
                    |value, cx| {
                        let secs = (value.clamp(1.0, 1440.0) as u64) * 60;
                        AppState::update_config(cx, |config| config.sync.interval_secs = secs);
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
                    |on, cx| match launch::set_enabled(&paths::launch_agent_path(), on) {
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
                            let path = paths::backups_dir(&state.data_dir)
                                .join(format!("kanso-{}.db", kanso_core::now_ms()));
                            let result =
                                std::fs::create_dir_all(paths::backups_dir(&state.data_dir))
                                    .map_err(anyhow::Error::from)
                                    .and_then(|()| {
                                        state.store.backup_to(&path).map_err(anyhow::Error::from)
                                    });
                            match result {
                                Ok(()) => window.push_notification(
                                    Notification::success(format!(
                                        "Backed up to {}",
                                        path.display()
                                    )),
                                    cx,
                                ),
                                Err(err) => window.push_notification(
                                    Notification::error(format!("Backup failed: {err:#}")),
                                    cx,
                                ),
                            }
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
