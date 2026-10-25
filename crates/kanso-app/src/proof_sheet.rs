//! The proof sheet in front of every change in Settings › Account ›
//! Sign-in methods (docs/ACCOUNTS.md › Account page › Signed in, ›
//! Changing a way in needs a fresh proof). It offers the proofs this
//! account can make on this device for the change, preferring a passkey,
//! then the password, then a provider; asks which when there is more than
//! one; then shows one form with the proof's field (the password, or a
//! security key's PIN), the change's own fields and the recovery key when
//! the change needs it. The task is `account_flow::change_method`.
//!
//! Which proofs a change takes, and when it needs the recovery key, are the
//! pure [`proof_options`] and [`needs_recovery`].

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_sync::{AccountMethods, PasskeyMethods, Provider, RecoveryKey};

use crate::account_flow::{self, ChangeRequest, MethodChange, PasskeyMethod, ProofWith};
use crate::account_words::{self as words, check_current_password, check_new_password};
use crate::dialogs::{self, Described, Field, FormSpec, Submit};
use crate::passkey_ceremony::{self, Platform};
use crate::security_key_flow;
use crate::services::{AccountState, Services};

/// A change the Sign-in methods section (or Security, for a passwordless
/// account) offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    AddPasskey,
    RemovePasskey(Vec<u8>),
    Link(Provider),
    Unlink(Provider),
    SetPassword,
    NewRecoveryKey,
    DeleteAccount,
}

/// One proof the sheet can offer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofOption {
    /// The platform's passkey sheet, or (Linux) a security key.
    Passkey {
        security_key: bool,
    },
    Password,
    Provider(Provider),
}

/// What this device can prove for the signed-in account.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ways {
    pub has_password: bool,
    /// The account has a passkey, and this device can run a ceremony for
    /// the server's relying party: `Some(true)` through a security key.
    pub passkey: Option<bool>,
    /// The account's linked providers this app can run.
    pub linked: Vec<Provider>,
}

/// The proofs `action` takes on this device, in the order offered: a
/// passkey, the password, then a provider. A provider proof only unlinks
/// its own provider, or (with the recovery key) sets a password, makes a
/// new key or deletes; it never adds a way in or removes a passkey.
pub fn proof_options(action: &Action, ways: &Ways) -> Vec<ProofOption> {
    let mut options = Vec::new();
    if let Some(security_key) = ways.passkey {
        options.push(ProofOption::Passkey { security_key });
    }
    if ways.has_password {
        options.push(ProofOption::Password);
    }
    match action {
        Action::Unlink(provider) if ways.linked.contains(provider) => {
            options.push(ProofOption::Provider(provider.clone()));
        }
        Action::SetPassword | Action::NewRecoveryKey | Action::DeleteAccount => {
            options.extend(ways.linked.iter().cloned().map(ProofOption::Provider));
        }
        _ => {}
    }
    options.truncate(3);
    options
}

/// Whether `action` with `proof` also needs the recovery key: a provider
/// proof for a password, a new key or a deletion, and removing the
/// account's last way in.
pub fn needs_recovery(action: &Action, proof: &ProofOption, methods: &AccountMethods) -> bool {
    let provider = matches!(proof, ProofOption::Provider(_));
    match action {
        Action::SetPassword | Action::NewRecoveryKey | Action::DeleteAccount => provider,
        Action::RemovePasskey(_) => {
            !methods.has_password && methods.linked.is_empty() && methods.passkeys.len() == 1
        }
        Action::Unlink(provider) => methods.last_method(provider),
        Action::AddPasskey | Action::Link(_) => false,
    }
}

/// Whether removing what `action` names leaves the account no other way in.
pub fn is_last_method(action: &Action, methods: &AccountMethods) -> bool {
    matches!(action, Action::RemovePasskey(_) | Action::Unlink(_))
        && needs_recovery(action, &ProofOption::Password, methods)
}

/// The server's passkeys as the signed-in page last read them.
fn advertised(cx: &App) -> Option<PasskeyMethods> {
    cx.global::<Services>().account_methods.passkeys.clone()
}

/// Whether this device runs the platform's passkey sheet for `server`.
pub fn platform_passkeys(passkeys: Option<&PasskeyMethods>, server: &str, cx: &App) -> bool {
    let services = cx.global::<Services>();
    let available = !services.platform_passkeys_failed
        && services
            .platform_passkeys
            .unwrap_or_else(passkey_ceremony::platform_available);
    passkey_ceremony::offers_platform_passkeys(Platform::current(), passkeys, server, available)
}

/// Whether this device can add or use a passkey for the account's server:
/// `Some(false)` for the platform's sheet, `Some(true)` for a Linux security
/// key, `None` when it can't.
pub fn ceremony(cx: &App) -> Option<bool> {
    let services = cx.global::<Services>();
    let server = services.session()?.server.clone();
    let passkeys = advertised(cx);
    if platform_passkeys(passkeys.as_ref(), &server, cx) {
        Some(false)
    } else if security_key_flow::offers_security_keys(
        Platform::current(),
        passkeys.as_ref(),
        &server,
    ) {
        Some(true)
    } else {
        None
    }
}

/// This account's ways in, as this device can prove them.
fn ways(methods: &AccountMethods, cx: &App) -> Ways {
    let runnable = |provider: &Provider| {
        matches!(
            provider,
            Provider::Apple | Provider::Google | Provider::Github
        )
    };
    Ways {
        has_password: methods.has_password,
        passkey: (!methods.passkeys.is_empty())
            .then(|| ceremony(cx))
            .flatten(),
        linked: methods
            .linked
            .iter()
            .map(|linked| linked.provider.clone())
            .filter(runnable)
            .collect(),
    }
}

fn username(cx: &App) -> Option<String> {
    match &cx.try_global::<Services>()?.account {
        AccountState::SignedIn { username } => Some(username.clone()),
        _ => None,
    }
}

pub fn provider_name(provider: &Provider) -> String {
    match provider {
        Provider::Apple => "Apple".to_owned(),
        Provider::Google => "Google".to_owned(),
        Provider::Github => "GitHub".to_owned(),
        Provider::Other(name) => name.clone(),
    }
}

/// Opens the sheet for `action`: the choice of proof when there is more
/// than one, then its form.
pub fn open(action: Action, window: &mut Window, cx: &mut App) {
    let Some(methods) = cx
        .global::<Services>()
        .account_methods
        .last_successful
        .clone()
    else {
        window.push_notification(Notification::info(words::METHODS_NOT_READY), cx);
        return;
    };
    let options = proof_options(&action, &ways(&methods, cx));
    match options.as_slice() {
        [] => {
            // A provider-only account sets a password first.
            let words = if methods.has_password || !methods.passkeys.is_empty() {
                words::NO_PROOF_HERE
            } else {
                words::SET_PASSWORD_FIRST
            };
            if methods.has_password || action == Action::SetPassword {
                window.push_notification(Notification::info(words), cx);
            } else {
                dialogs::confirm(
                    "Confirm it's you",
                    words,
                    "Set a password…",
                    window,
                    cx,
                    |window, cx| open(Action::SetPassword, window, cx),
                );
            }
        }
        [only] => form(action, only.clone(), &methods, window, cx),
        _ => {
            let choices = options.iter().map(described).collect();
            dialogs::choose(
                "Confirm it's you",
                words::PROOF_CHOICE,
                choices,
                window,
                cx,
                move |picked, window, cx| {
                    if let Some(option) = picked.and_then(|ix| options.get(ix)) {
                        form(action.clone(), option.clone(), &methods, window, cx);
                    }
                },
            );
        }
    }
}

fn described(option: &ProofOption) -> Described {
    let (label, description) = match option {
        ProofOption::Passkey {
            security_key: false,
        } => (
            "Use a passkey".to_owned(),
            "Your device's passkey sheet confirms it's you.".to_owned(),
        ),
        ProofOption::Passkey { security_key: true } => (
            "Use a security key".to_owned(),
            "Touch your security key when it blinks.".to_owned(),
        ),
        ProofOption::Password => (
            "Use your password".to_owned(),
            "Type your current password.".to_owned(),
        ),
        ProofOption::Provider(provider) => {
            let name = provider_name(provider);
            (
                format!("Sign in with {name}"),
                format!("Kanso opens {name} to confirm it's you."),
            )
        }
    };
    Described {
        label: label.into(),
        description: description.into(),
    }
}

/// The title, button and description of `action`'s form.
fn wording(
    action: &Action,
    security_key: bool,
    last: bool,
    username: &str,
) -> (String, String, String) {
    let last_words = if last {
        format!(" {}", words::LAST_METHOD_WARNING)
    } else {
        String::new()
    };
    match action {
        Action::AddPasskey if security_key => (
            "Add a security key".into(),
            "Add security key".into(),
            words::SECURITY_KEY_SIGN_UP_NOTE.into(),
        ),
        Action::AddPasskey => (
            "Add a passkey".into(),
            "Add passkey".into(),
            "Your device makes a new passkey for this account, and says whether it can unlock a new device.".into(),
        ),
        Action::RemovePasskey(_) => (
            "Remove this passkey?".into(),
            "Remove".into(),
            format!("It no longer signs in to this account. Devices already signed in keep their notes.{last_words}"),
        ),
        Action::Link(provider) => {
            let name = provider_name(provider);
            (
                format!("Link {name}"),
                format!("Link {name}"),
                format!("After this, Kanso opens {name}. Sign in there to link it to this account."),
            )
        }
        Action::Unlink(provider) => {
            let name = provider_name(provider);
            (
                format!("Unlink {name}?"),
                "Unlink".into(),
                format!("{name} no longer signs in to this account.{last_words}"),
            )
        }
        Action::SetPassword => (
            "Set a password".into(),
            "Set password".into(),
            words::SET_PASSWORD_TEXT.into(),
        ),
        Action::NewRecoveryKey => (
            "New recovery key".into(),
            "Make a new key".into(),
            "The key you saved stops working, and Kanso shows the new one to save instead.".into(),
        ),
        Action::DeleteAccount => (
            "Delete account?".into(),
            "Delete account".into(),
            words::delete_account_words(username),
        ),
    }
}

/// What the form asks for, in order.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Input {
    Typed,
    CurrentPassword,
    Pin,
    NewPassword,
    Confirm,
    Recovery,
}

/// The form for `action` with `proof`.
fn form(
    action: Action,
    proof: ProofOption,
    methods: &AccountMethods,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(username) = username(cx) else {
        return;
    };
    let security_key = matches!(proof, ProofOption::Passkey { security_key: true })
        || (action == Action::AddPasskey && ceremony(cx) == Some(true));
    let recovery = needs_recovery(&action, &proof, methods);
    let last = is_last_method(&action, methods);
    let (title, submit, mut description) = wording(&action, security_key, last, &username);
    match &proof {
        ProofOption::Passkey { .. } if action != Action::AddPasskey => {
            description.push_str(" Your passkey confirms it's you first.");
        }
        ProofOption::Provider(provider) => {
            description.push_str(&format!(
                " Kanso opens {} to confirm it's you.",
                provider_name(provider)
            ));
        }
        _ => {}
    }
    let mut spec = FormSpec::new(title, submit).description(description);
    let mut inputs = Vec::new();
    if action == Action::DeleteAccount {
        // confirm_typed puts its field first.
        inputs.push(Input::Typed);
        spec = spec.danger();
    }
    if proof == ProofOption::Password {
        spec = spec.field(Field::secret("Current password"));
        inputs.push(Input::CurrentPassword);
    }
    if security_key {
        spec = spec.field(Field::secret("Security key PIN").hint(words::SECURITY_KEY_PIN_HINT));
        inputs.push(Input::Pin);
    }
    if action == Action::SetPassword {
        spec = spec
            .field(
                Field::secret("New password")
                    .hint("At least 12 characters. A few random words work well."),
            )
            .field(Field::secret("Confirm new password"));
        inputs.extend([Input::NewPassword, Input::Confirm]);
    }
    if recovery {
        spec = spec.field(Field::text("Recovery key").hint(words::RECOVERY_FOR_PROOF_HINT));
        inputs.push(Input::Recovery);
    }
    if matches!(action, Action::RemovePasskey(_) | Action::Unlink(_)) {
        spec = spec.danger();
    }
    let deleting = action == Action::DeleteAccount;
    let typed = username.clone();
    let on_submit = move |values: Vec<String>, window: &mut Window, cx: &mut App| match request(
        &action, &proof, &inputs, &values, &username, window, cx,
    ) {
        Ok(request) => Submit::Running(account_flow::change_method(request, cx)),
        Err(words) => Submit::Invalid(words),
    };
    if deleting {
        dialogs::confirm_typed(spec, &typed, window, cx, on_submit);
    } else {
        dialogs::form(spec, window, cx, on_submit);
    }
}

/// The task's request from the form's values, or the words for what's wrong.
fn request(
    action: &Action,
    proof: &ProofOption,
    inputs: &[Input],
    values: &[String],
    username: &str,
    window: &mut Window,
    cx: &App,
) -> Result<ChangeRequest, String> {
    let value = |input: Input| {
        inputs
            .iter()
            .position(|i| *i == input)
            .and_then(|ix| values.get(ix))
            .map(String::as_str)
    };
    let recovery = value(Input::Recovery)
        .map(|text| {
            RecoveryKey::parse(text).map_err(|err| words::recovery_key_words(err).to_owned())
        })
        .transpose()?;
    let change = match action {
        Action::AddPasskey => MethodChange::AddPasskey,
        Action::RemovePasskey(id) => MethodChange::RemovePasskey {
            credential_id: id.clone(),
        },
        Action::Link(provider) => MethodChange::Link(provider.clone()),
        Action::Unlink(provider) => MethodChange::Unlink(provider.clone()),
        Action::SetPassword => MethodChange::SetPassword(check_new_password(
            value(Input::NewPassword).unwrap_or_default(),
            value(Input::Confirm).unwrap_or_default(),
            username,
        )?),
        Action::NewRecoveryKey => MethodChange::NewRecoveryKey,
        Action::DeleteAccount => MethodChange::DeleteAccount {
            typed: value(Input::Typed).unwrap_or_default().to_owned(),
        },
    };
    let proof_with = match proof {
        ProofOption::Password => ProofWith::Password(check_current_password(
            value(Input::CurrentPassword).unwrap_or_default(),
            username,
        )?),
        ProofOption::Passkey { .. } => ProofWith::Passkey,
        ProofOption::Provider(provider) => ProofWith::Provider(provider.clone()),
    };
    let needs_ceremony =
        matches!(proof, ProofOption::Passkey { .. }) || *action == Action::AddPasskey;
    let passkey = if !needs_ceremony {
        None
    } else if inputs.contains(&Input::Pin) {
        let keys = security_keys(cx);
        let Some(device) = security_key_flow::pick(&keys).cloned() else {
            return Err(words::CONNECT_SECURITY_KEY.to_owned());
        };
        let pin = value(Input::Pin).unwrap_or_default();
        Some(PasskeyMethod::SecurityKey {
            device,
            pin: (!pin.is_empty()).then(|| zeroize::Zeroizing::new(pin.to_owned())),
        })
    } else {
        Some(PasskeyMethod::Platform {
            window: passkey_ceremony::native_window(window),
        })
    };
    Ok(ChangeRequest {
        change,
        proof: proof_with,
        recovery,
        passkey,
    })
}

/// The security keys attached now: the ones the page enumerated, or a
/// fresh look (none off Linux).
fn security_keys(cx: &App) -> Vec<kanso_fido::SecurityKeyDevice> {
    let known = &cx.global::<Services>().security_keys;
    if known.is_empty() {
        security_key_flow::enumerate()
    } else {
        known.clone()
    }
}

#[cfg(test)]
mod tests {
    use kanso_sync::provider::{LinkedProvider, PasskeyInfo};
    use kanso_sync::{AccountMethods, Provider};
    use proptest::prelude::*;

    use super::{Action, ProofOption, Ways, is_last_method, needs_recovery, proof_options};

    fn methods(has_password: bool, linked: &[Provider], passkeys: usize) -> AccountMethods {
        AccountMethods {
            has_password,
            rp_id: Some("sync.kansoapp.app".into()),
            available: vec![Provider::Apple, Provider::Google, Provider::Github],
            linked: linked
                .iter()
                .cloned()
                .map(|provider| LinkedProvider {
                    provider,
                    created_at: 1,
                })
                .collect(),
            passkeys: (0..passkeys)
                .map(|n| PasskeyInfo {
                    credential_id: vec![n as u8],
                    created_at: 1,
                    last_used_at: None,
                    prf: true,
                    transports: vec![],
                })
                .collect(),
        }
    }

    fn providers() -> impl Strategy<Value = Vec<Provider>> {
        proptest::sample::subsequence(
            vec![Provider::Apple, Provider::Google, Provider::Github],
            0..=3,
        )
    }

    fn actions() -> impl Strategy<Value = Action> {
        proptest::sample::select(vec![
            Action::AddPasskey,
            Action::RemovePasskey(vec![0]),
            Action::Link(Provider::Google),
            Action::Unlink(Provider::Apple),
            Action::Unlink(Provider::Github),
            Action::SetPassword,
            Action::NewRecoveryKey,
            Action::DeleteAccount,
        ])
    }

    #[test]
    fn a_passkey_comes_first_then_the_password_then_a_provider() {
        let ways = Ways {
            has_password: true,
            passkey: Some(false),
            linked: vec![Provider::Apple],
        };
        assert_eq!(
            proof_options(&Action::Unlink(Provider::Apple), &ways),
            vec![
                ProofOption::Passkey {
                    security_key: false
                },
                ProofOption::Password,
                ProofOption::Provider(Provider::Apple),
            ]
        );
        assert_eq!(
            proof_options(&Action::AddPasskey, &ways),
            vec![
                ProofOption::Passkey {
                    security_key: false
                },
                ProofOption::Password
            ]
        );
        let provider_only = Ways {
            linked: vec![Provider::Github],
            ..Ways::default()
        };
        assert!(proof_options(&Action::Link(Provider::Apple), &provider_only).is_empty());
        assert_eq!(
            proof_options(&Action::SetPassword, &provider_only),
            vec![ProofOption::Provider(Provider::Github)]
        );
    }

    #[test]
    fn the_last_way_in_and_a_provider_proof_ask_for_the_recovery_key() {
        let only_passkey = methods(false, &[], 1);
        assert!(needs_recovery(
            &Action::RemovePasskey(vec![0]),
            &ProofOption::Passkey {
                security_key: false
            },
            &only_passkey
        ));
        assert!(is_last_method(
            &Action::RemovePasskey(vec![0]),
            &only_passkey
        ));
        let only_apple = methods(false, &[Provider::Apple], 0);
        assert!(is_last_method(
            &Action::Unlink(Provider::Apple),
            &only_apple
        ));
        assert!(!is_last_method(
            &Action::Unlink(Provider::Apple),
            &methods(true, &[Provider::Apple], 0)
        ));
        assert!(needs_recovery(
            &Action::NewRecoveryKey,
            &ProofOption::Provider(Provider::Apple),
            &only_apple
        ));
        assert!(!needs_recovery(
            &Action::NewRecoveryKey,
            &ProofOption::Password,
            &methods(true, &[], 0)
        ));
    }

    /// A provider proof never adds a way in, removes a passkey or unlinks
    /// another provider; it never comes before a passkey or the password; at
    /// most three are offered.
    fn check_options(action: Action, ways: Ways) -> Result<(), TestCaseError> {
        let options = proof_options(&action, &ways);
        prop_assert!(options.len() <= 3);
        for (ix, option) in options.iter().enumerate() {
            if let ProofOption::Provider(provider) = option {
                prop_assert!(ways.linked.contains(provider));
                let allowed = match &action {
                    Action::Unlink(unlinked) => unlinked == provider,
                    Action::SetPassword | Action::NewRecoveryKey | Action::DeleteAccount => true,
                    _ => false,
                };
                prop_assert!(allowed, "{action:?} with {provider:?}");
                prop_assert!(
                    options[ix..]
                        .iter()
                        .all(|o| matches!(o, ProofOption::Provider(_)))
                );
            }
        }
        let strong = usize::from(ways.has_password) + usize::from(ways.passkey.is_some());
        prop_assert_eq!(
            options
                .iter()
                .filter(|o| !matches!(o, ProofOption::Provider(_)))
                .count(),
            strong
        );
        Ok(())
    }

    /// A provider proof for a password, a new key or a deletion always takes
    /// the recovery key, and so does removing the last way in.
    fn check_recovery(
        action: Action,
        has_password: bool,
        linked: Vec<Provider>,
        passkeys: usize,
        provider_proof: bool,
    ) -> Result<(), TestCaseError> {
        let account = methods(has_password, &linked, passkeys);
        let proof = if provider_proof {
            ProofOption::Provider(linked.first().cloned().unwrap_or(Provider::Apple))
        } else {
            ProofOption::Password
        };
        let ways_in = usize::from(has_password) + linked.len() + passkeys;
        let expected = match &action {
            Action::SetPassword | Action::NewRecoveryKey | Action::DeleteAccount => provider_proof,
            Action::RemovePasskey(_) => passkeys == 1 && ways_in == 1,
            Action::Unlink(provider) => ways_in == 1 && linked.contains(provider),
            _ => false,
        };
        prop_assert_eq!(needs_recovery(&action, &proof, &account), expected);
        Ok(())
    }

    proptest! {
        #[test]
        fn prop_a_provider_proof_only_does_what_the_server_allows(
            action in actions(),
            ways in (any::<bool>(), proptest::option::of(any::<bool>()), providers()),
        ) {
            let (has_password, passkey, linked) = ways;
            check_options(action, Ways { has_password, passkey, linked })?;
        }

        #[test]
        fn prop_the_recovery_key_is_asked_for_exactly_when_needed(
            action in actions(),
            account in (any::<bool>(), providers(), 0usize..3, any::<bool>()),
        ) {
            let (has_password, linked, passkeys, provider_proof) = account;
            check_recovery(action, has_password, linked, passkeys, provider_proof)?;
        }
    }
}
