//! The account half of a passkey or security-key sign-in, behind one narrow
//! seam, [`PasskeyAccounts`] (docs/ACCOUNTS.md › Passkeys).
//!
//! The ceremony is the platform's ([`crate::passkey_ceremony`]) or the Linux
//! CTAP2 client's ([`crate::security_key_flow`]); both arrive here as
//! `kanso-sync`'s [`PasskeyCeremony`]. What happens around it — asking the
//! server for a challenge, sending the assertion back, and creating an
//! account — belongs to `kanso-sync`.
//!
//! **Switch point.** `kanso-sync` is gaining `sign_in_with_passkey` and
//! `sign_up_with_passkey` (sign-in PR B). Until it has them,
//! [`ApiPasskeyAccounts`] drives the routes `kanso_sync::Api` already has:
//! sign-in works and leaves the device locked, so device approval or the
//! recovery key unlocks it (the same path as a provider sign-in); sign-up
//! answers [`PasskeyFlowError::SignUpNotReady`]. When PR B lands, the two
//! method bodies become calls to those functions, and a PRF wrap that opens
//! becomes [`PasskeySignIn::Unlocked`]. Nothing else in the app changes.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use kanso_sync::api::{PasskeyFinishRequest, PasskeyStartRequest};
use kanso_sync::passkey::{PasskeyAssertionChallenge, PasskeyCeremony, PasskeyCeremonyError};
use kanso_sync::{AccountError, Backend, DeviceIdentity, ProviderSession, Step};

use crate::passkey_ceremony::{OFFICIAL_RP_ID, ceremony_origin, rp_id_fits_server};

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
}

impl RpRule {
    pub fn allows(self, rp_id: &str, server: &str) -> bool {
        match self {
            RpRule::Entitlement => rp_id == OFFICIAL_RP_ID,
            RpRule::ServerHost => rp_id_fits_server(rp_id, server),
        }
    }
}

/// A finished passkey sign-in.
pub enum PasskeySignIn {
    /// Signed in, without the data key: the device waits for approval or
    /// the recovery key, as after a provider sign-in. `could_unlock` says the
    /// credential returned a PRF output and the account holds its wrap, so
    /// `kanso-sync`'s passkey unlock (PR B) will open it here.
    Locked {
        session: ProviderSession,
        could_unlock: bool,
    },
}

/// A finished passkey sign-up: whether the new credential can unlock a new
/// device. Only PR B's `sign_up_with_passkey` produces one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PasskeySignUp {
    pub can_unlock: bool,
}

/// Why a passkey sign-in or sign-up didn't finish.
#[derive(Debug)]
pub enum PasskeyFlowError {
    Account(AccountError),
    Ceremony(PasskeyCeremonyError),
    /// The server doesn't advertise passkeys (any more).
    NotAdvertised,
    /// The server's relying party isn't one this app may use for it.
    WrongRelyingParty,
    /// A security key asked for its PIN, which this attempt didn't carry.
    PinRequired,
    /// Creating an account with a passkey needs `kanso-sync`'s
    /// `sign_up_with_passkey` (PR B).
    SignUpNotReady,
}

impl From<AccountError> for PasskeyFlowError {
    fn from(error: AccountError) -> Self {
        PasskeyFlowError::Account(error)
    }
}

impl From<kanso_sync::ApiError> for PasskeyFlowError {
    fn from(error: kanso_sync::ApiError) -> Self {
        PasskeyFlowError::Account(error.into())
    }
}

/// The seam between the app's passkey flows and `kanso-sync`. `Send + Sync`:
/// it is called on an account task's thread.
pub trait PasskeyAccounts: Send + Sync {
    /// Signs in with a passkey. `username` is given for a security key
    /// (username-first, so the server lists the account's credentials as
    /// `allowList`) and absent for the platform picker.
    #[allow(clippy::too_many_arguments, reason = "the ceremony's whole context")]
    fn sign_in(
        &self,
        backend: &dyn Backend,
        server: &str,
        username: Option<&str>,
        device: &LoginDevice,
        rule: RpRule,
        ceremony: &dyn PasskeyCeremony,
        progress: &dyn Fn(Step),
    ) -> Result<PasskeySignIn, PasskeyFlowError>;

    /// Creates an account whose first way in is a new passkey.
    #[allow(clippy::too_many_arguments, reason = "the ceremony's whole context")]
    fn sign_up(
        &self,
        backend: &dyn Backend,
        server: &str,
        username: &str,
        device: &DeviceIdentity,
        rule: RpRule,
        ceremony: &dyn PasskeyCeremony,
        progress: &dyn Fn(Step),
    ) -> Result<PasskeySignUp, PasskeyFlowError>;
}

/// Today's [`PasskeyAccounts`]: the passkey routes of `kanso_sync::Api`.
pub struct ApiPasskeyAccounts;

fn decode(text: &str) -> Result<Vec<u8>, PasskeyFlowError> {
    URL_SAFE_NO_PAD
        .decode(text)
        .map_err(|_| AccountError::Protocol("the server sent a malformed passkey value".into()).into())
}

fn encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

impl PasskeyAccounts for ApiPasskeyAccounts {
    fn sign_in(
        &self,
        backend: &dyn Backend,
        server: &str,
        username: Option<&str>,
        device: &LoginDevice,
        rule: RpRule,
        ceremony: &dyn PasskeyCeremony,
        progress: &dyn Fn(Step),
    ) -> Result<PasskeySignIn, PasskeyFlowError> {
        let server = kanso_sync::check_endpoint(server).map_err(AccountError::Endpoint)?;
        let username = username
            .map(kanso_sync::normalize_username)
            .transpose()
            .map_err(AccountError::from)?;
        progress(Step::Contacting);
        let api = backend.api(&server);
        let advertised = api
            .health()?
            .auth
            .and_then(|auth| auth.passkeys)
            .ok_or(PasskeyFlowError::NotAdvertised)?;
        if !rule.allows(&advertised.rp_id, &server) {
            return Err(PasskeyFlowError::WrongRelyingParty);
        }
        let start = api.passkey_start(&PasskeyStartRequest {
            username: username.clone(),
            device: device.clone(),
        })?;
        if start.rp_id != advertised.rp_id {
            return Err(AccountError::Protocol(
                "the server started a passkey sign-in for another relying party".into(),
            )
            .into());
        }
        let challenge = PasskeyAssertionChallenge {
            rp_id: start.rp_id.clone(),
            origin: ceremony_origin(&start.rp_id, &advertised.origins),
            challenge: decode(&start.challenge)?,
            allow_credentials: start
                .allow_credentials
                .iter()
                .map(|credential| decode(&credential.id))
                .collect::<Result<_, _>>()?,
            request_prf: true,
        };
        let assertion = ceremony
            .assert(&challenge)
            .map_err(PasskeyFlowError::Ceremony)?;
        progress(Step::Contacting);
        let answer = api.passkey_finish(&PasskeyFinishRequest {
            flow_id: start.flow_id,
            credential_id: encode(&assertion.credential_id),
            authenticator_data: encode(&assertion.authenticator_data),
            client_data_json: encode(&assertion.client_data_json),
            signature: encode(&assertion.signature),
            user_handle: assertion.user_handle.as_deref().map(encode),
            device: device.clone(),
        })?;
        if username
            .as_deref()
            .is_some_and(|name| name != answer.username)
        {
            return Err(AccountError::Protocol(
                "the server answered for another account".into(),
            )
            .into());
        }
        let could_unlock = assertion.prf.is_some() && answer.passkey_wrapped_key.is_some();
        Ok(PasskeySignIn::Locked {
            session: ProviderSession {
                account_id: answer.account_id,
                username: answer.username,
                token: answer.token,
                key_id: answer.key_id,
                epoch: answer.epoch,
                has_password: answer.has_password,
                passkey_wrapped_key: answer.passkey_wrapped_key,
                evicted_device: answer.evicted_device,
            },
            could_unlock,
        })
    }

    fn sign_up(
        &self,
        _backend: &dyn Backend,
        _server: &str,
        _username: &str,
        _device: &DeviceIdentity,
        _rule: RpRule,
        _ceremony: &dyn PasskeyCeremony,
        _progress: &dyn Fn(Step),
    ) -> Result<PasskeySignUp, PasskeyFlowError> {
        // Creating the account needs a new data key, its recovery wrap and a
        // session, which only `kanso-sync` can make: PR B's
        // `sign_up_with_passkey`.
        Err(PasskeyFlowError::SignUpNotReady)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use kanso_sync::api::{NewDevice, PasskeySignupFinishRequest, PasskeySignupStartRequest};
    use kanso_sync::crypto::{DataKey, WrapPurpose, key_id, recovery_keys, wrap};
    use kanso_sync::passkey::PasskeyRegistrationChallenge;
    use kanso_sync::{MemoryServer, PasskeyAssertion, PasskeyRegistration, RecoveryKey};

    use super::*;

    const SERVER: &str = "https://memory.kanso.test";
    /// `MemoryServer`'s relying party.
    const MEMORY_PASSKEY_RP_ID: &str = "memory.kanso.test";

    /// A ceremony that records what it was asked and answers with a fixed
    /// credential, as a platform sheet would.
    struct Fake {
        credential_id: Vec<u8>,
        prf: Option<[u8; 32]>,
        asked: RefCell<Vec<PasskeyAssertionChallenge>>,
        fail: Option<PasskeyCeremonyError>,
    }

    impl Fake {
        fn new(credential_id: Vec<u8>) -> Fake {
            Fake {
                credential_id,
                prf: Some([5; 32]),
                asked: RefCell::new(Vec::new()),
                fail: None,
            }
        }
    }

    impl PasskeyCeremony for Fake {
        fn register(
            &self,
            _challenge: &PasskeyRegistrationChallenge,
        ) -> Result<PasskeyRegistration, PasskeyCeremonyError> {
            Err(PasskeyCeremonyError::Unavailable)
        }

        fn assert(
            &self,
            challenge: &PasskeyAssertionChallenge,
        ) -> Result<PasskeyAssertion, PasskeyCeremonyError> {
            self.asked.borrow_mut().push(challenge.clone());
            if let Some(error) = self.fail {
                return Err(error);
            }
            Ok(PasskeyAssertion {
                credential_id: self.credential_id.clone(),
                authenticator_data: vec![1; 37],
                client_data_json: kanso_sync::passkey::client_data_json(
                    "webauthn.get",
                    &challenge.challenge,
                    &challenge.origin,
                )
                .to_vec(),
                signature: vec![2; 64],
                user_handle: None,
                prf: self.prf,
            })
        }
    }

    /// An account whose only way in is the passkey `credential_id`, made
    /// through the server's own passkey sign-up routes.
    fn passkey_account(server: &MemoryServer, username: &str, credential_id: &[u8]) {
        let api = server.api(SERVER);
        let account_id = format!("a_{}", "1".repeat(32));
        let device = LoginDevice::ephemeral();
        let new_device = NewDevice {
            id: device.id.clone(),
            sealed_name: None,
            proof: device.proof.clone(),
        };
        let start = api
            .passkey_signup_start(&PasskeySignupStartRequest {
                username: username.into(),
                account_id: account_id.clone(),
                device: new_device.clone(),
            })
            .unwrap();
        let data_key = DataKey::generate();
        let recovery = recovery_keys(&RecoveryKey::generate(), SERVER, username);
        api.passkey_signup_finish(&PasskeySignupFinishRequest {
            flow_id: start.flow_id,
            username: username.into(),
            account_id: account_id.clone(),
            credential_id: encode(credential_id),
            attestation_object: encode(&[0xa3]),
            client_data_json: encode(b"{}"),
            transports: vec!["internal".into()],
            passkey_wrapped_key: Some(wrap(
                &recovery.kek,
                WrapPurpose::Passkey,
                &account_id,
                &data_key,
            )),
            recovery_auth_key: recovery.auth_key.to_base64url(),
            recovery_wrapped_key: wrap(
                &recovery.kek,
                WrapPurpose::Recovery,
                &account_id,
                &data_key,
            ),
            key_id: key_id(&data_key),
            device: new_device,
        })
        .unwrap();
    }

    #[test]
    fn a_platform_passkey_signs_in_and_leaves_the_device_locked() {
        let server = MemoryServer::new();
        passkey_account(&server, "quiet-otter-4821", &[7; 16]);
        let ceremony = Fake::new(vec![7; 16]);
        let steps = RefCell::new(Vec::new());
        let signed = ApiPasskeyAccounts
            .sign_in(
                &server,
                SERVER,
                None,
                &LoginDevice::ephemeral(),
                RpRule::ServerHost,
                &ceremony,
                &|step| steps.borrow_mut().push(step),
            )
            .unwrap();
        let PasskeySignIn::Locked {
            session,
            could_unlock,
        } = signed;
        assert_eq!(session.username, "quiet-otter-4821");
        assert!(!session.has_password);
        assert!(could_unlock, "a PRF output and a wrap");
        // The platform picker: no username, so no allowList, and PRF asked.
        let asked = ceremony.asked.borrow();
        assert_eq!(asked.len(), 1);
        assert_eq!(asked[0].rp_id, MEMORY_PASSKEY_RP_ID);
        assert_eq!(asked[0].origin, "https://memory.kanso.test");
        assert_eq!(asked[0].challenge.len(), 32);
        assert!(asked[0].allow_credentials.is_empty());
        assert!(asked[0].request_prf);
        assert!(steps.borrow().contains(&Step::Contacting));
    }

    #[test]
    fn a_security_key_signs_in_username_first_with_the_accounts_credentials() {
        let server = MemoryServer::new();
        passkey_account(&server, "quiet-otter-4821", &[7; 16]);
        let mut ceremony = Fake::new(vec![7; 16]);
        ceremony.prf = None;
        let signed = ApiPasskeyAccounts
            .sign_in(
                &server,
                SERVER,
                Some(" Quiet-Otter-4821 "),
                &LoginDevice::ephemeral(),
                RpRule::ServerHost,
                &ceremony,
                &|_| {},
            )
            .unwrap();
        let PasskeySignIn::Locked { could_unlock, .. } = signed;
        assert!(!could_unlock, "no hmac-secret output, no unlock");
        assert_eq!(
            ceremony.asked.borrow()[0].allow_credentials,
            vec![vec![7u8; 16]]
        );
    }

    #[test]
    fn a_failed_ceremony_or_an_unknown_credential_does_not_sign_in() {
        let server = MemoryServer::new();
        passkey_account(&server, "quiet-otter-4821", &[7; 16]);
        let mut ceremony = Fake::new(vec![7; 16]);
        ceremony.fail = Some(PasskeyCeremonyError::Cancelled);
        let sign_in = |ceremony: &Fake| {
            ApiPasskeyAccounts
                .sign_in(
                    &server,
                    SERVER,
                    None,
                    &LoginDevice::ephemeral(),
                    RpRule::ServerHost,
                    ceremony,
                    &|_| {},
                )
                .err()
        };
        assert!(matches!(
            sign_in(&ceremony),
            Some(PasskeyFlowError::Ceremony(PasskeyCeremonyError::Cancelled))
        ));
        assert!(matches!(
            sign_in(&Fake::new(vec![8; 16])),
            Some(PasskeyFlowError::Account(AccountError::InvalidCredentials))
        ));
    }

    #[test]
    fn a_relying_party_outside_the_rule_never_reaches_the_ceremony() {
        let server = MemoryServer::new();
        let ceremony = Fake::new(vec![7; 16]);
        let result = ApiPasskeyAccounts.sign_in(
            &server,
            SERVER,
            None,
            &LoginDevice::ephemeral(),
            RpRule::Entitlement,
            &ceremony,
            &|_| {},
        );
        assert!(matches!(
            result.err(),
            Some(PasskeyFlowError::WrongRelyingParty)
        ));
        assert!(ceremony.asked.borrow().is_empty());
    }

    #[test]
    fn rules_follow_the_entitlement_and_the_server_host() {
        assert!(RpRule::Entitlement.allows(OFFICIAL_RP_ID, "https://kanso.antonlabs.cc"));
        assert!(!RpRule::Entitlement.allows("sync.example.com", "https://sync.example.com"));
        assert!(RpRule::ServerHost.allows("sync.example.com", "https://sync.example.com"));
        assert!(!RpRule::ServerHost.allows(OFFICIAL_RP_ID, "https://sync.example.com"));
    }
}
