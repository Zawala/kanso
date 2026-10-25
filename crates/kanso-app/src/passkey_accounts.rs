//! The account half of a passkey or security-key sign-in, behind one narrow
//! seam, [`PasskeyAccounts`] (docs/ACCOUNTS.md › Passkeys, › Account flows ›
//! Sign in with a passkey, › Sign up with a passkey).
//!
//! The ceremony is the platform's ([`crate::passkey_ceremony`]) or the Linux
//! CTAP2 client's ([`crate::security_key_flow`]); both arrive here as
//! `kanso-sync`'s [`PasskeyCeremony`]. Everything around it (the server's
//! challenge, the assertion or attestation sent back, the PRF unlock, a new
//! account's keys) is `kanso-sync`'s: [`SyncPasskeyAccounts`] calls its
//! `start_passkey_sign_in`, `sign_in_with_passkey`, `start_passkey_sign_up`
//! and `sign_up_with_passkey`. Between the start and the ceremony this module
//! checks the relying party against the platform's rule, so a ceremony is
//! never run for a relying party the app may not use.

use kanso_sync::passkey::{PasskeyCeremony, PasskeyCeremonyError};
use kanso_sync::{
    AccountError, Backend, DeviceIdentity, PasskeyAssertion, PasskeyRegistration, PasskeySignIn,
    PasskeySignInStart, PasskeySignUpStart, RotationError, SecretStore, SignedUp, Step,
};

use crate::passkey_ceremony::{OFFICIAL_RP_ID, rp_id_fits_server};

pub use kanso_sync::api::LoginDevice;

/// Which relying parties a ceremony may be run for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RpRule {
    /// macOS platform passkeys: only the RP ID in the app's Associated
    /// Domains entitlement.
    Entitlement,
    /// Windows platform passkeys and Linux security keys: the server's host
    /// or a parent of it.
    ServerHost,
    /// Tests against `MemoryServer`, whose relying party isn't the
    /// server's host.
    #[cfg(test)]
    Any,
}

impl RpRule {
    /// The rule for this platform's platform passkeys.
    pub const fn platform() -> RpRule {
        if cfg!(target_os = "macos") {
            RpRule::Entitlement
        } else {
            RpRule::ServerHost
        }
    }

    pub fn allows(self, rp_id: &str, server: &str) -> bool {
        match self {
            RpRule::Entitlement => rp_id == OFFICIAL_RP_ID,
            RpRule::ServerHost => rp_id_fits_server(rp_id, server),
            #[cfg(test)]
            RpRule::Any => true,
        }
    }
}

/// A finished passkey sign-up, and whether the new credential can unlock a
/// new device (the platform answered PRF, so its wrap was sent).
pub struct PasskeySignedUp {
    pub signed: SignedUp,
    pub can_unlock: bool,
}

/// A finished passkey sign-in, and whether the credential answered PRF.
pub struct PasskeySignedIn {
    pub signed: PasskeySignIn,
    pub prf: bool,
}

/// Why a passkey sign-in or sign-up didn't finish.
#[derive(Debug)]
pub enum PasskeyFlowError {
    Account(AccountError),
    /// A sign-up's request failed; its recovery key may need showing.
    Rotation(RotationError),
    Ceremony(PasskeyCeremonyError),
    /// The server's relying party isn't one this app may use for it.
    WrongRelyingParty,
}

impl From<AccountError> for PasskeyFlowError {
    fn from(error: AccountError) -> Self {
        PasskeyFlowError::Account(error)
    }
}

impl From<RotationError> for PasskeyFlowError {
    fn from(error: RotationError) -> Self {
        PasskeyFlowError::Rotation(error)
    }
}

/// The seam between the app's passkey flows and `kanso-sync`. `Send +
/// Sync`: it is called on an account task's thread.
pub trait PasskeyAccounts: Send + Sync {
    fn start_sign_in(
        &self,
        backend: &dyn Backend,
        server: &str,
        username: Option<&str>,
        login_device: &LoginDevice,
    ) -> Result<PasskeySignInStart, AccountError>;

    fn sign_in(
        &self,
        backend: &dyn Backend,
        secrets: &dyn SecretStore,
        start: &PasskeySignInStart,
        device: &DeviceIdentity,
        assertion: PasskeyAssertion,
        progress: &dyn Fn(Step),
    ) -> Result<PasskeySignIn, AccountError>;

    fn start_sign_up(
        &self,
        backend: &dyn Backend,
        server: &str,
        username: &str,
        device: &DeviceIdentity,
    ) -> Result<PasskeySignUpStart, AccountError>;

    fn sign_up(
        &self,
        backend: &dyn Backend,
        secrets: &dyn SecretStore,
        start: &PasskeySignUpStart,
        device: &DeviceIdentity,
        registration: PasskeyRegistration,
        progress: &dyn Fn(Step),
    ) -> Result<SignedUp, RotationError>;
}

/// [`PasskeyAccounts`] through `kanso-sync`'s passkey flows.
pub struct SyncPasskeyAccounts;

impl PasskeyAccounts for SyncPasskeyAccounts {
    fn start_sign_in(
        &self,
        backend: &dyn Backend,
        server: &str,
        username: Option<&str>,
        login_device: &LoginDevice,
    ) -> Result<PasskeySignInStart, AccountError> {
        kanso_sync::start_passkey_sign_in(backend, server, username, login_device)
    }

    fn sign_in(
        &self,
        backend: &dyn Backend,
        secrets: &dyn SecretStore,
        start: &PasskeySignInStart,
        device: &DeviceIdentity,
        assertion: PasskeyAssertion,
        progress: &dyn Fn(Step),
    ) -> Result<PasskeySignIn, AccountError> {
        kanso_sync::sign_in_with_passkey(backend, secrets, start, device, assertion, progress)
    }

    fn start_sign_up(
        &self,
        backend: &dyn Backend,
        server: &str,
        username: &str,
        device: &DeviceIdentity,
    ) -> Result<PasskeySignUpStart, AccountError> {
        kanso_sync::start_passkey_sign_up(backend, server, username, device)
    }

    fn sign_up(
        &self,
        backend: &dyn Backend,
        secrets: &dyn SecretStore,
        start: &PasskeySignUpStart,
        device: &DeviceIdentity,
        registration: PasskeyRegistration,
        progress: &dyn Fn(Step),
    ) -> Result<SignedUp, RotationError> {
        kanso_sync::sign_up_with_passkey(backend, secrets, start, device, registration, progress)
    }
}

/// What a sign-in or sign-up needs besides its ceremony.
pub struct PasskeyContext<'a> {
    pub accounts: &'a dyn PasskeyAccounts,
    pub backend: &'a dyn Backend,
    pub secrets: &'a dyn SecretStore,
    pub server: &'a str,
    pub device: &'a DeviceIdentity,
    pub rule: RpRule,
    pub progress: &'a dyn Fn(Step),
}

/// Signs in with a passkey. `username` is given for a security key
/// (username-first, so the server lists the account's credentials as
/// `allowList`) and absent for the platform picker. The flow is bound to a
/// new ephemeral device: before the assertion this library doesn't know
/// which account it is for.
pub fn sign_in(
    cx: &PasskeyContext<'_>,
    username: Option<&str>,
    ceremony: &dyn PasskeyCeremony,
) -> Result<PasskeySignedIn, PasskeyFlowError> {
    (cx.progress)(Step::Contacting);
    let login_device = LoginDevice::ephemeral();
    let start = cx
        .accounts
        .start_sign_in(cx.backend, cx.server, username, &login_device)?;
    if !cx.rule.allows(&start.challenge.rp_id, &start.server) {
        return Err(PasskeyFlowError::WrongRelyingParty);
    }
    let assertion = ceremony
        .assert(&start.challenge)
        .map_err(PasskeyFlowError::Ceremony)?;
    let prf = assertion.prf().is_some();
    let signed = cx.accounts.sign_in(
        cx.backend,
        cx.secrets,
        &start,
        cx.device,
        assertion,
        cx.progress,
    )?;
    Ok(PasskeySignedIn { signed, prf })
}

/// Creates an account whose first way in is a new passkey.
pub fn sign_up(
    cx: &PasskeyContext<'_>,
    username: &str,
    ceremony: &dyn PasskeyCeremony,
) -> Result<PasskeySignedUp, PasskeyFlowError> {
    (cx.progress)(Step::Contacting);
    let start = cx
        .accounts
        .start_sign_up(cx.backend, cx.server, username, cx.device)?;
    if !cx.rule.allows(&start.challenge.rp_id, &start.server) {
        return Err(PasskeyFlowError::WrongRelyingParty);
    }
    let registration = ceremony
        .register(&start.challenge)
        .map_err(PasskeyFlowError::Ceremony)?;
    let can_unlock = registration.prf().is_some();
    let signed = cx.accounts.sign_up(
        cx.backend,
        cx.secrets,
        &start,
        cx.device,
        registration,
        cx.progress,
    )?;
    Ok(PasskeySignedUp { signed, can_unlock })
}

/// Words a, b and four digits, as the Worker suggests for a provider sign-up
/// (`quiet-otter-4821`): the passkey form's editable suggestion.
pub fn suggest_username(seed: u64) -> String {
    const ADJECTIVES: &[&str] = &[
        "amber", "bold", "brave", "bright", "calm", "clever", "cosy", "crisp", "daring", "eager",
        "fair", "gentle", "glad", "golden", "happy", "honest", "jolly", "keen", "kind", "lively",
        "lucky", "mellow", "merry", "misty", "noble", "patient", "proud", "quick", "quiet",
        "rapid", "silver", "steady", "sunny", "swift", "tidy", "warm", "wise", "witty",
    ];
    const ANIMALS: &[&str] = &[
        "badger", "beaver", "bison", "crane", "deer", "dolphin", "eagle", "falcon", "ferret",
        "finch", "fox", "gecko", "heron", "ibis", "koala", "lark", "lemur", "lynx", "marten",
        "moose", "newt", "otter", "owl", "panda", "puffin", "quail", "raven", "robin", "seal",
        "sparrow", "stoat", "swan", "tapir", "tern", "tiger", "turtle", "walrus", "wren",
    ];
    let mut rng = fastrand::Rng::with_seed(seed);
    format!(
        "{}-{}-{}",
        ADJECTIVES[rng.usize(..ADJECTIVES.len())],
        ANIMALS[rng.usize(..ANIMALS.len())],
        rng.u16(1000..10000)
    )
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::Arc;

    use kanso_sync::passkey::{PasskeyAssertionChallenge, PasskeyRegistrationChallenge};
    use kanso_sync::{MemorySecrets, MemoryServer, TestAuthenticator, normalize_username};
    use proptest::prelude::*;

    use super::*;

    const SERVER: &str = "https://memory.kanso.test";
    /// `MemoryServer`'s relying party.
    const MEMORY_PASSKEY_RP_ID: &str = "memory.kanso.test";

    /// A software passkey that records what it was asked, as a platform
    /// sheet would see it, and can be made to fail.
    struct Recorded {
        key: TestAuthenticator,
        asked: RefCell<Vec<PasskeyAssertionChallenge>>,
        registered: RefCell<Vec<PasskeyRegistrationChallenge>>,
        fail: Option<PasskeyCeremonyError>,
    }

    impl Recorded {
        fn new(key: TestAuthenticator) -> Recorded {
            Recorded {
                key,
                asked: RefCell::new(Vec::new()),
                registered: RefCell::new(Vec::new()),
                fail: None,
            }
        }
    }

    impl PasskeyCeremony for Recorded {
        fn register(
            &self,
            challenge: &PasskeyRegistrationChallenge,
        ) -> Result<PasskeyRegistration, PasskeyCeremonyError> {
            self.registered.borrow_mut().push(challenge.clone());
            match self.fail {
                Some(error) => Err(error),
                None => self.key.register(challenge),
            }
        }

        fn assert(
            &self,
            challenge: &PasskeyAssertionChallenge,
        ) -> Result<PasskeyAssertion, PasskeyCeremonyError> {
            self.asked.borrow_mut().push(challenge.clone());
            match self.fail {
                Some(error) => Err(error),
                None => self.key.assert(challenge),
            }
        }
    }

    struct World {
        server: Arc<MemoryServer>,
        secrets: MemorySecrets,
        device: DeviceIdentity,
    }

    impl World {
        fn new() -> World {
            World::on(Arc::new(MemoryServer::new()))
        }

        /// Another library, on another device, for the same server.
        fn on(server: Arc<MemoryServer>) -> World {
            World {
                server,
                secrets: MemorySecrets::default(),
                device: DeviceIdentity::from_parts(
                    &format!("l_{}", "4".repeat(32)),
                    "This Mac",
                    std::array::from_fn(|_| fastrand::u8(..)),
                ),
            }
        }

        fn context<'a>(&'a self, rule: RpRule, progress: &'a dyn Fn(Step)) -> PasskeyContext<'a> {
            PasskeyContext {
                accounts: &SyncPasskeyAccounts,
                backend: self.server.as_ref(),
                secrets: &self.secrets,
                server: SERVER,
                device: &self.device,
                rule,
                progress,
            }
        }
    }

    #[test]
    fn a_passkey_sign_up_then_sign_in_unlocks_with_its_prf() {
        let world = World::new();
        let steps = RefCell::new(Vec::new());
        let progress = |step| steps.borrow_mut().push(step);
        let ceremony = Recorded::new(TestAuthenticator::new(1));
        let created = sign_up(
            &world.context(RpRule::ServerHost, &progress),
            "quiet-otter-4821",
            &ceremony,
        )
        .unwrap();
        assert!(created.can_unlock);
        assert_eq!(created.signed.session.username, "quiet-otter-4821");
        let registered = ceremony.registered.borrow();
        assert_eq!(registered[0].rp_id, MEMORY_PASSKEY_RP_ID);
        assert!(registered[0].request_prf);
        assert!(steps.borrow().contains(&Step::Contacting));

        // Another device: the PRF wrap opens the key.
        let elsewhere = World::on(world.server.clone());
        let signed = sign_in(
            &elsewhere.context(RpRule::ServerHost, &|_| {}),
            None,
            &ceremony,
        )
        .unwrap();
        assert!(signed.prf);
        assert!(matches!(signed.signed, PasskeySignIn::Unlocked(_)));
        // The platform picker: no username, so no allowList, and PRF asked.
        let asked = ceremony.asked.borrow();
        assert!(asked[0].allow_credentials.is_empty());
        assert!(asked[0].request_prf);
    }

    #[test]
    fn a_key_without_prf_signs_up_and_then_signs_in_locked() {
        let world = World::new();
        let ceremony = Recorded::new(TestAuthenticator::new(2).without_prf());
        let created = sign_up(
            &world.context(RpRule::ServerHost, &|_| {}),
            "quiet-otter-4821",
            &ceremony,
        )
        .unwrap();
        assert!(!created.can_unlock);
        let elsewhere = World::on(world.server.clone());
        // Username-first, as a security key signs in.
        let signed = sign_in(
            &elsewhere.context(RpRule::ServerHost, &|_| {}),
            Some(" Quiet-Otter-4821 "),
            &ceremony,
        )
        .unwrap();
        assert!(!signed.prf);
        assert!(matches!(signed.signed, PasskeySignIn::Locked(_)));
        assert_eq!(
            ceremony.asked.borrow()[0].allow_credentials,
            vec![ceremony.key.credential_id().to_vec()]
        );
    }

    #[test]
    fn a_failed_ceremony_creates_and_signs_in_nothing() {
        let world = World::new();
        let mut ceremony = Recorded::new(TestAuthenticator::new(3));
        ceremony.fail = Some(PasskeyCeremonyError::Cancelled);
        let context = world.context(RpRule::ServerHost, &|_| {});
        assert!(matches!(
            sign_up(&context, "quiet-otter-4821", &ceremony).err(),
            Some(PasskeyFlowError::Ceremony(PasskeyCeremonyError::Cancelled))
        ));
        assert!(matches!(
            sign_in(&context, None, &ceremony).err(),
            Some(PasskeyFlowError::Ceremony(PasskeyCeremonyError::Cancelled))
        ));
        // A credential the server doesn't know is refused.
        assert!(matches!(
            sign_in(&context, None, &Recorded::new(TestAuthenticator::new(4))).err(),
            Some(PasskeyFlowError::Account(_))
        ));
    }

    #[test]
    fn a_relying_party_outside_the_rule_never_reaches_the_ceremony() {
        let world = World::new();
        let ceremony = Recorded::new(TestAuthenticator::new(5));
        let context = world.context(RpRule::Entitlement, &|_| {});
        assert!(matches!(
            sign_in(&context, None, &ceremony).err(),
            Some(PasskeyFlowError::WrongRelyingParty)
        ));
        assert!(matches!(
            sign_up(&context, "quiet-otter-4821", &ceremony).err(),
            Some(PasskeyFlowError::WrongRelyingParty)
        ));
        assert!(ceremony.asked.borrow().is_empty());
        assert!(ceremony.registered.borrow().is_empty());
    }

    #[test]
    fn rules_follow_the_entitlement_and_the_server_host() {
        assert!(RpRule::Entitlement.allows(OFFICIAL_RP_ID, "https://kanso.antonlabs.cc"));
        assert!(!RpRule::Entitlement.allows("sync.example.com", "https://sync.example.com"));
        assert!(RpRule::ServerHost.allows("sync.example.com", "https://sync.example.com"));
        assert!(!RpRule::ServerHost.allows(OFFICIAL_RP_ID, "https://sync.example.com"));
    }

    proptest! {
        /// Every suggestion is a valid username that normalizes to itself.
        #[test]
        fn prop_suggestions_are_valid_usernames(seed: u64) {
            let name = suggest_username(seed);
            prop_assert_eq!(normalize_username(&name).unwrap(), name.clone());
            prop_assert!(kanso_sync::check_new_username(&name).is_ok(), "{}", name);
        }
    }
}
