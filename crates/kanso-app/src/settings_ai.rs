//! Settings › AI: whether AI is on, where questions go, and the key that
//! cloud providers need. The key is kept in the Keychain, never in
//! `config.json`.

use std::sync::Arc;

use gpui_kit::component::Disableable as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable as _, v_flex};
use gpui_kit::*;
use kanso_ai::{ClientSettings, Provider};
use kanso_sync::SecretStore;

use crate::config::AiSettings;
use crate::services::Services;
use crate::settings::{Row, section};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The Keychain item a provider's API key is kept in.
pub fn api_key_item(provider: Provider) -> String {
    format!("ai-key:{}", provider.key())
}

/// The API key saved for `provider`, if one is.
pub fn read_key(cx: &App, provider: Provider) -> Option<String> {
    let secrets = secrets(cx)?;
    read_from(secrets.as_ref(), provider)
}

/// Saves `key` for `provider` in the Keychain.
pub fn write_key(cx: &App, provider: Provider, key: &str) -> Result<(), String> {
    let secrets =
        secrets(cx).ok_or_else(|| "The Keychain isn't available in this build.".to_owned())?;
    write_to(secrets.as_ref(), provider, key)
}

fn secrets(cx: &App) -> Option<Arc<dyn SecretStore>> {
    cx.try_global::<Services>()?.secrets.clone()
}

fn read_from(secrets: &dyn SecretStore, provider: Provider) -> Option<String> {
    secrets
        .get(&api_key_item(provider))
        .ok()
        .flatten()
        .map(|key| key.to_string())
        .filter(|key| !key.trim().is_empty())
}

fn write_to(secrets: &dyn SecretStore, provider: Provider, key: &str) -> Result<(), String> {
    secrets
        .set(&api_key_item(provider), key.trim())
        .map_err(|err| err.to_string())
}

/// The plain line about where a question goes: the server it reaches, and
/// what is never sent. Settings and the Ask panel show the same line.
pub fn requests_line(provider: Provider, base_url: &str) -> String {
    let host = provider.host(base_url);
    if provider.needs_key() {
        format!(
            "Questions and the note excerpts they use go straight from this Mac to {host}. Nothing goes through Kanso's servers. Locked notes and notes tagged #ai-exclude are never sent."
        )
    } else {
        format!(
            "Questions and the note excerpts they use go straight from this Mac to the model at {host}. Nothing goes through Kanso's servers. Locked notes and notes tagged #ai-exclude are never sent."
        )
    }
}

/// The client settings the fields describe.
fn client_settings(ai: &AiSettings, typed_key: &str, saved_key: Option<&str>) -> ClientSettings {
    let typed = typed_key.trim();
    let api_key = if typed.is_empty() {
        saved_key.map(str::to_owned)
    } else {
        Some(typed.to_owned())
    };
    ClientSettings {
        provider: ai.provider,
        base_url: ai.base_url.clone(),
        model: ai.model.clone(),
        api_key,
    }
}

/// What the Test button found, or is doing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Test {
    #[default]
    NotRun,
    Running,
    Ok(String),
    Failed(String),
}

pub struct AiPage {
    base_url: Entity<InputState>,
    model: Entity<InputState>,
    key: Entity<InputState>,
    /// Whether a key is in the Keychain for the chosen provider.
    has_key: bool,
    test: Test,
    _subscriptions: Vec<Subscription>,
    task: Option<Task<()>>,
}

impl AiPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let ai = cx.global::<AppState>().config.ai.clone();
        let mut field = |value: &str, placeholder: &'static str, cx: &mut Context<Self>| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder)
                    .default_value(value)
            })
        };
        let base_url = field(&ai.base_url, "http://localhost:11434/v1", cx);
        let model = field(&ai.model, "Model name", cx);
        let key = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder("Paste the key")
        });
        let subscriptions = [&base_url, &model, &key]
            .into_iter()
            .map(|input| {
                cx.subscribe_in(
                    input,
                    window,
                    |this: &mut AiPage, _, event: &InputEvent, window, cx| match event {
                        InputEvent::PressEnter { shift: false, .. } => {
                            this.commit(window, cx);
                        }
                        InputEvent::Blur => this.commit(window, cx),
                        InputEvent::Change => {
                            this.test = Test::NotRun;
                            cx.notify();
                        }
                        _ => {}
                    },
                )
            })
            .collect();
        AiPage {
            base_url,
            model,
            key,
            has_key: read_key(cx, ai.provider).is_some(),
            test: Test::NotRun,
            _subscriptions: subscriptions,
            task: None,
        }
    }

    /// The page came into view: says whether a key is saved for the provider
    /// chosen now.
    pub fn shown(&mut self, cx: &mut Context<Self>) {
        let provider = cx.global::<AppState>().config.ai.provider;
        let has_key = read_key(cx, provider).is_some();
        if has_key != self.has_key {
            self.has_key = has_key;
            cx.notify();
        }
    }

    /// Turns AI on or off. Menus and any open Ask panel follow.
    pub fn set_enabled(&mut self, on: bool, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.ai.enabled = on);
        crate::settings_ai::ai_changed(cx);
        if !on {
            self.task = None;
            self.test = Test::NotRun;
        }
        cx.notify();
    }

    /// Picks `provider`, with the address and model it starts with.
    fn set_provider(&mut self, provider: Provider, window: &mut Window, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.ai.choose(provider));
        let ai = cx.global::<AppState>().config.ai.clone();
        self.base_url.update(cx, |field, cx| {
            field.set_value(ai.base_url.as_str(), window, cx)
        });
        self.model.update(cx, |field, cx| {
            field.set_value(ai.model.as_str(), window, cx)
        });
        self.key
            .update(cx, |field, cx| field.set_value("", window, cx));
        self.has_key = read_key(cx, provider).is_some();
        self.test = Test::NotRun;
        cx.notify();
    }

    /// Writes what the fields say into the config, and saves a typed key in
    /// the Keychain.
    fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let base_url = self.base_url.read(cx).value().trim().to_owned();
        let model = self.model.read(cx).value().trim().to_owned();
        let typed_key = self.key.read(cx).value().trim().to_owned();
        let provider = cx.global::<AppState>().config.ai.provider;
        AppState::update_config(cx, |config| {
            if !base_url.is_empty() {
                config.ai.base_url = base_url;
            }
            config.ai.model = model;
        });
        if !typed_key.is_empty() {
            match write_key(cx, provider, &typed_key) {
                Ok(()) => {
                    self.has_key = true;
                    self.key
                        .update(cx, |field, cx| field.set_value("", window, cx));
                }
                Err(err) => {
                    self.test = Test::Failed(format!("Couldn't save the key: {err}"));
                }
            }
        }
        cx.notify();
    }

    /// Sends a tiny request to check the settings, off the main thread.
    fn run_test(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.test == Test::Running {
            return;
        }
        self.commit(window, cx);
        let ai = cx.global::<AppState>().config.ai.clone();
        let typed_key = self.key.read(cx).value().trim().to_owned();
        let saved = read_key(cx, ai.provider);
        let settings = client_settings(&ai, &typed_key, saved.as_deref());
        self.test = Test::Running;
        cx.notify();
        let probe = cx
            .background_executor()
            .spawn(async move { kanso_ai::probe(&settings) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = probe.await;
            this.update(cx, |this, cx| {
                this.task = None;
                this.test = match result {
                    Ok(said) => Test::Ok(format!("OK: {said}")),
                    Err(err) => Test::Failed(err.to_string()),
                };
                cx.notify();
            })
            .ok();
        }));
    }

    fn provider_row(&self, ai: &AiSettings, cx: &mut Context<Self>) -> Row {
        let view = cx.entity();
        let chosen = ai.provider;
        let provider = Button::new("ai-provider")
            .debug_selector(|| "ai-provider".into())
            .outline()
            .small()
            .label(ai.provider.label())
            .dropdown_caret(true)
            .dropdown_menu(move |mut menu, _, _| {
                for option in Provider::ALL {
                    let view = view.clone();
                    menu = menu.item(
                        PopupMenuItem::new(option.label())
                            .checked(option == chosen)
                            .on_click(move |_, window, cx| {
                                view.update(cx, |this, cx| this.set_provider(option, window, cx));
                            }),
                    );
                }
                menu
            });
        Row::new(gpui_kit::assets::IconName::Cpu, "Provider")
            .detail(provider_detail(ai.provider))
            .trailing(provider)
    }

    fn base_url_row(&self, ai: &AiSettings) -> Row {
        let editable = matches!(ai.provider, Provider::LocalModel | Provider::Custom);
        let row = Row::new(gpui_kit::assets::IconName::Link, "Server address");
        if editable {
            row.detail(if ai.provider == Provider::Custom {
                "The OpenAI-compatible server to ask, for example http://localhost:1234/v1."
            } else {
                "Ollama's OpenAI-compatible address. Change it for a model on another machine."
            })
            .trailing(
                div()
                    .w(px(FIELD_WIDTH))
                    .debug_selector(|| "ai-base-url".into())
                    .child(Input::new(&self.base_url)),
            )
        } else {
            row.value(ai.base_url.clone())
        }
    }

    fn model_row(&self, ai: &AiSettings) -> Row {
        let placeholder = if ai.provider == Provider::Custom {
            "The model's name at that server"
        } else {
            "The model to ask"
        };
        Row::new(gpui_kit::assets::IconName::Bot, "Model")
            .detail(placeholder)
            .trailing(
                div()
                    .w(px(FIELD_WIDTH))
                    .debug_selector(|| "ai-model".into())
                    .child(Input::new(&self.model)),
            )
    }

    fn key_row(&self) -> Row {
        let detail = if self.has_key {
            "Saved in the Keychain. Type a new key to replace it."
        } else {
            "Kept in the Keychain, never in the settings file."
        };
        Row::new(gpui_kit::assets::IconName::Key, "API key")
            .detail(detail)
            .trailing(
                div()
                    .w(px(FIELD_WIDTH))
                    .debug_selector(|| "ai-api-key".into())
                    .child(Input::new(&self.key).mask_toggle()),
            )
    }

    fn test_row(&self, ai: &AiSettings, view: Entity<Self>) -> Row {
        let detail = match &self.test {
            Test::NotRun => {
                "Sends one short question to check the address, model and key.".to_owned()
            }
            Test::Running => "Asking the model…".to_owned(),
            Test::Ok(said) => said.clone(),
            Test::Failed(why) => why.clone(),
        };
        let running = self.test == Test::Running;
        Row::new(gpui_kit::assets::IconName::FlaskConical, "Test")
            .detail(detail)
            .trailing(
                Button::new("ai-test")
                    .debug_selector(|| "ai-test".into())
                    .outline()
                    .small()
                    .label(if running { "Testing…" } else { "Test" })
                    .disabled(running || !ai.enabled)
                    .on_click({
                        let view = view.clone();
                        move |_, window, cx| {
                            let view = view.clone();
                            view.update(cx, |this, cx| this.run_test(window, cx));
                        }
                    }),
            )
    }
}

/// What a provider is, in a line.
fn provider_detail(provider: Provider) -> &'static str {
    match provider {
        Provider::LocalModel => {
            "Ollama on this Mac, through its OpenAI-compatible address. No key, and the notes stay on this Mac."
        }
        Provider::OpenAi => "OpenAI's own models, with a key from your OpenAI account.",
        Provider::Anthropic => "Claude models, with a key from your Anthropic account.",
        Provider::OpenRouter => "Many providers through OpenRouter, with a key from your account.",
        Provider::Custom => {
            "Another OpenAI-compatible server, such as LM Studio, at an address you type."
        }
    }
}

/// Turns the AI on or off everywhere it is visible: the View menu, and an
/// Ask panel that was left open.
pub fn ai_changed(cx: &mut App) {
    // The click is inside a window's update; the windows and the menus are
    // reached once it has finished.
    cx.defer(|cx| {
        let enabled = cx
            .try_global::<AppState>()
            .is_some_and(|state| state.config.ai.enabled);
        crate::workspace::set_menus(cx);
        crate::workspace::for_each_workspace(cx, |workspace, cx| {
            workspace.inspector.update(cx, |inspector, cx| {
                if !enabled {
                    inspector.stop_ask(cx);
                    if inspector.tab == crate::config::InspectorTab::Ask {
                        inspector.set_tab(crate::config::InspectorTab::Outline, cx);
                    }
                }
            });
        });
    });
}

/// The width of a page's fields, as Account's are.
const FIELD_WIDTH: f32 = 260.;

impl Render for AiPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let ai = cx.global::<AppState>().config.ai.clone();
        let view = cx.entity();
        let mut rows = vec![
            Row::new(gpui_kit::assets::IconName::Sparkles, "AI")
                .detail(if ai.enabled {
                    "Answers come from your own notes, with links to the notes they used."
                } else {
                    "Off. Nothing asks a model, and nothing about AI shows, until you turn this on."
                })
                .trailing(div().debug_selector(|| "ai-enabled".into()).child(
                    Switch::new("ai-enabled").checked(ai.enabled).on_click({
                        let view = view.clone();
                        move |on, _, cx| {
                            view.update(cx, |this, cx| this.set_enabled(*on, cx));
                        }
                    }),
                )),
        ];
        if ai.enabled {
            rows.push(self.provider_row(&ai, cx));
            rows.push(self.base_url_row(&ai));
            rows.push(self.model_row(&ai));
            if ai.provider.needs_key() {
                rows.push(self.key_row());
            }
            rows.push(self.test_row(&ai, view.clone()));
        }
        let mut page = v_flex().w_full().child(section("AI", rows, colors));
        if ai.enabled {
            page = page.child(
                div()
                    .mt(px(12.))
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(colors.text_secondary)
                    .child(requests_line(ai.provider, &ai.base_url)),
            );
        }
        page
    }
}

#[cfg(test)]
mod tests {
    use super::{Test, api_key_item, client_settings, read_from, requests_line, write_to};
    use crate::config::AiSettings;
    use kanso_ai::Provider;
    use kanso_sync::keychain::MemorySecrets;
    use proptest::prelude::*;

    #[test]
    fn each_provider_keeps_its_key_apart() {
        assert_eq!(api_key_item(Provider::OpenAi), "ai-key:openai");
        assert_eq!(api_key_item(Provider::Anthropic), "ai-key:anthropic");
        assert_ne!(
            api_key_item(Provider::OpenRouter),
            api_key_item(Provider::OpenAi)
        );
    }

    #[test]
    fn a_key_goes_to_the_keychain_and_comes_back() {
        let secrets = MemorySecrets::default();
        assert_eq!(read_from(&secrets, Provider::Anthropic), None);
        write_to(&secrets, Provider::Anthropic, "  sk-ant-1  ").unwrap();
        assert_eq!(
            read_from(&secrets, Provider::Anthropic).as_deref(),
            Some("sk-ant-1")
        );
        // Another provider has no key of its own.
        assert_eq!(read_from(&secrets, Provider::OpenAi), None);
    }

    #[test]
    fn a_typed_key_is_used_before_the_saved_one() {
        let ai = AiSettings::for_provider(Provider::OpenAi);
        assert_eq!(
            client_settings(&ai, "typed", Some("saved"))
                .api_key
                .as_deref(),
            Some("typed")
        );
        assert_eq!(
            client_settings(&ai, "  ", Some("saved")).api_key.as_deref(),
            Some("saved")
        );
        assert_eq!(client_settings(&ai, "", None).api_key, None);
        assert_eq!(Test::default(), Test::NotRun);
    }

    #[test]
    fn the_line_names_the_server_and_the_exclusions() {
        let line = requests_line(Provider::Anthropic, "https://api.anthropic.com");
        assert!(line.contains("api.anthropic.com"), "{line}");
        assert!(line.contains("#ai-exclude"), "{line}");
        assert!(line.contains("Locked notes"), "{line}");
        assert!(line.contains("Kanso's servers"), "{line}");
    }

    proptest! {
        /// The line always names the server the address points at, says
        /// locked notes and `#ai-exclude` are never sent, and has no line
        /// breaks.
        #[test]
        fn prop_the_line_always_says_the_same(
            provider in prop::sample::select(Provider::ALL.to_vec()),
            host in "[a-z0-9.-]{1,30}",
            scheme in prop::sample::select(vec!["http", "https"]),
        ) {
            let address = format!("{scheme}://{host}/v1");
            let line = requests_line(provider, &address);
            prop_assert!(line.contains(&host));
            prop_assert!(line.contains("#ai-exclude"));
            prop_assert!(line.contains("Locked notes"));
            prop_assert!(!line.contains('\n'));
        }
    }
}
