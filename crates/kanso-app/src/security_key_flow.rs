//! Linux security keys for the account flows (docs/ACCOUNTS.md › Passkeys ›
//! Where the ceremony runs › Linux).
//!
//! Linux has no platform passkey UI: the app is the CTAP2 client, through
//! `kanso-fido` (libfido2). This module decides whether the signed-out page
//! offers a security key, picks the attached key, and implements
//! `kanso-sync`'s [`SecurityKeyCeremony`] over it. `kanso-sync`'s
//! [`SecurityKeyPasskeys`] turns that into the [`PasskeyCeremony`] the
//! account flows in [`crate::passkey_accounts`] take, so they never see which
//! kind of authenticator ran.
//!
//! libfido2's make-credential binding gives no `hmac-secret-mc` output, so a
//! registration on a key that advertises `hmac-secret` is followed by one
//! immediate local assertion over a challenge derived from the registration
//! challenge, user verification required (a second touch). `kanso-sync`'s
//! `register_security_key` verifies it against the new credential's public
//! key before its PRF output may wrap the data key; otherwise the credential
//! registers without a wrap and the app says it signs in only.

use std::sync::atomic::{AtomicBool, Ordering};

use kanso_fido::{FidoError, SecurityKeyDevice};
use kanso_sync::PasskeyMethods;
use kanso_sync::passkey::{
    PasskeyAssertionChallenge, PasskeyCeremony, PasskeyCeremonyError, PasskeyRegistrationChallenge,
};
use kanso_sync::security_key::{
    SecurityKeyAssertion, SecurityKeyCeremony, SecurityKeyPasskeys, SecurityKeyRegistration,
};
use zeroize::Zeroizing;

use crate::passkey_ceremony::Platform;

/// Whether the signed-out page shows [Continue with a security key] (or
/// its Create twin): Linux only, and only when the server advertises
/// security keys for a relying party that is its own host or a parent of it.
/// Whether a key is attached decides if the button works or the page asks
/// for one ([`crate::account_words::CONNECT_SECURITY_KEY`]).
pub fn offers_security_keys(
    platform: Platform,
    passkeys: Option<&PasskeyMethods>,
    server: &str,
) -> bool {
    platform == Platform::Linux
        && passkeys.is_some_and(|passkeys| {
            passkeys.security_key && kanso_sync::passkey_rp_allowed(&passkeys.rp_id, server)
        })
}

/// The key a ceremony uses: the first that advertises `hmac-secret`, so the
/// credential can unlock, else the first attached.
pub fn pick(devices: &[SecurityKeyDevice]) -> Option<&SecurityKeyDevice> {
    devices
        .iter()
        .find(|device| device.hmac_secret)
        .or_else(|| devices.first())
}

/// The attached keys (none off Linux).
pub fn enumerate() -> Vec<SecurityKeyDevice> {
    kanso_fido::enumerate()
}

/// The libfido2 calls, behind a trait so the ceremony's logic is testable
/// without a key.
pub trait Fido: Send + Sync {
    fn register(
        &self,
        device: &SecurityKeyDevice,
        request: &kanso_fido::RegistrationRequest,
    ) -> Result<kanso_fido::Registration, FidoError>;

    fn assert(
        &self,
        device: &SecurityKeyDevice,
        request: &kanso_fido::AssertionRequest,
    ) -> Result<kanso_fido::Assertion, FidoError>;
}

/// [`Fido`] through `kanso-fido`.
pub struct LibFido;

impl Fido for LibFido {
    fn register(
        &self,
        device: &SecurityKeyDevice,
        request: &kanso_fido::RegistrationRequest,
    ) -> Result<kanso_fido::Registration, FidoError> {
        kanso_fido::register(device, request)
    }

    fn assert(
        &self,
        device: &SecurityKeyDevice,
        request: &kanso_fido::AssertionRequest,
    ) -> Result<kanso_fido::Assertion, FidoError> {
        kanso_fido::assert(device, request)
    }
}

/// One security key, with the PIN when it needs one (zeroizing, and never
/// logged). After a failure, [`Self::pin_required`] says whether the key
/// asked for a PIN.
pub struct FidoKey<'a> {
    fido: &'a dyn Fido,
    device: SecurityKeyDevice,
    pin: Option<Zeroizing<String>>,
    pin_required: AtomicBool,
}

impl<'a> FidoKey<'a> {
    pub fn new(
        fido: &'a dyn Fido,
        device: SecurityKeyDevice,
        pin: Option<Zeroizing<String>>,
    ) -> FidoKey<'a> {
        FidoKey {
            fido,
            device,
            pin: pin.filter(|pin| !pin.is_empty()),
            pin_required: AtomicBool::new(false),
        }
    }

    pub fn pin_required(&self) -> bool {
        self.pin_required.load(Ordering::Acquire)
    }

    /// This key as the account flows' ceremony.
    pub fn passkeys(&self) -> SecurityKeyPasskeys<'_> {
        SecurityKeyPasskeys {
            ceremony: self,
            hmac_secret: self.device.hmac_secret,
        }
    }

    /// `kanso-fido`'s error in `kanso-sync`'s words; a PIN request is
    /// remembered so the flow can ask for it.
    fn error(&self, error: FidoError) -> PasskeyCeremonyError {
        match error {
            FidoError::PinRequired => {
                self.pin_required.store(true, Ordering::Release);
                PasskeyCeremonyError::Failed
            }
            FidoError::NoDevice | FidoError::Invalid => PasskeyCeremonyError::Unavailable,
            FidoError::Refused(_) | FidoError::Incomplete => PasskeyCeremonyError::Failed,
        }
    }
}

impl SecurityKeyCeremony for FidoKey<'_> {
    /// `rk`, user verification required; `hmac-secret` is enabled on the
    /// credential when the key has it, and the PRF comes from the local
    /// assertion `kanso-sync` runs next.
    fn make_credential(
        &self,
        challenge: &PasskeyRegistrationChallenge,
    ) -> Result<SecurityKeyRegistration, PasskeyCeremonyError> {
        let made = self
            .fido
            .register(
                &self.device,
                &kanso_fido::RegistrationRequest {
                    rp_id: challenge.rp_id.clone(),
                    origin: challenge.origin.clone(),
                    challenge: challenge.challenge.clone(),
                    user_handle: challenge.user_handle.clone(),
                    username: challenge.username.clone(),
                    display_name: challenge.display_name.clone(),
                    hmac_secret: challenge.request_prf && self.device.hmac_secret,
                    pin: self.pin.clone(),
                },
            )
            .map_err(|error| self.error(error))?;
        if !kanso_sync::security_key::auth_data_has_credential_id(
            &made.auth_data,
            &made.credential_id,
        ) {
            return Err(PasskeyCeremonyError::Failed);
        }
        Ok(SecurityKeyRegistration {
            credential_id: made.credential_id.clone(),
            auth_data: made.auth_data.clone(),
            client_data_json: made.client_data_json.clone(),
            transports: made.transports.clone(),
            prf: None,
        })
    }

    /// User verification required; `hmac-secret` over `ctap_salt` when PRF
    /// is asked for.
    fn get_assertion(
        &self,
        challenge: &PasskeyAssertionChallenge,
    ) -> Result<SecurityKeyAssertion, PasskeyCeremonyError> {
        let assertion = self
            .fido
            .assert(
                &self.device,
                &kanso_fido::AssertionRequest {
                    rp_id: challenge.rp_id.clone(),
                    origin: challenge.origin.clone(),
                    challenge: challenge.challenge.clone(),
                    allow_credentials: challenge.allow_credentials.clone(),
                    hmac_secret: challenge.request_prf,
                    pin: self.pin.clone(),
                },
            )
            .map_err(|error| self.error(error))?;
        Ok(SecurityKeyAssertion {
            credential_id: assertion.credential_id.clone(),
            authenticator_data: assertion.authenticator_data.clone(),
            client_data_json: assertion.client_data_json.clone(),
            signature: assertion.signature.clone(),
            user_handle: assertion.user_handle.clone(),
            prf: assertion.hmac_secret,
        })
    }
}

/// A [`PasskeyCeremony`] that owns its key: what an account task's thread
/// builds from the page's choice.
pub struct OwnedSecurityKey {
    pub device: SecurityKeyDevice,
    pub pin: Option<Zeroizing<String>>,
}

impl OwnedSecurityKey {
    /// Runs `work` with this key as a ceremony, and says afterwards whether
    /// the key asked for its PIN.
    pub fn with<T>(self, work: impl FnOnce(&dyn PasskeyCeremony) -> T) -> (T, bool) {
        let key = FidoKey::new(&LibFido, self.device, self.pin);
        let out = work(&key.passkeys());
        (out, key.pin_required())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use kanso_sync::{MemorySecrets, MemoryServer, PasskeySignIn, TestAuthenticator};

    use super::*;
    use crate::passkey_accounts::{self, PasskeyContext, RpRule, SyncPasskeyAccounts};

    const SERVER: &str = kanso_sync::MEMORY_PASSKEY_SERVER;
    const RP_ID: &str = "memory.kanso.test";

    fn device(hmac_secret: bool) -> SecurityKeyDevice {
        SecurityKeyDevice {
            path: "/dev/hidraw3".into(),
            vendor: "1050".into(),
            product: "YubiKey".into(),
            hmac_secret,
        }
    }

    fn methods(rp_id: &str, security_key: bool) -> PasskeyMethods {
        PasskeyMethods {
            rp_id: rp_id.into(),
            security_key,
        }
    }

    /// libfido2 over a software credential: real signatures, so
    /// `kanso-sync`'s local check and `MemoryServer` verify them.
    struct FakeKey {
        key: TestAuthenticator,
        /// Signs local assertions with another credential's key.
        forged: Option<TestAuthenticator>,
        needs_pin: bool,
        registers: Mutex<Vec<kanso_fido::RegistrationRequest>>,
        asserts: Mutex<Vec<kanso_fido::AssertionRequest>>,
    }

    impl FakeKey {
        fn new(key: TestAuthenticator) -> FakeKey {
            FakeKey {
                key,
                forged: None,
                needs_pin: false,
                registers: Mutex::default(),
                asserts: Mutex::default(),
            }
        }
    }

    impl Fido for FakeKey {
        fn register(
            &self,
            _device: &SecurityKeyDevice,
            request: &kanso_fido::RegistrationRequest,
        ) -> Result<kanso_fido::Registration, FidoError> {
            self.registers.lock().unwrap().push(request.clone());
            if self.needs_pin && request.pin.is_none() {
                return Err(FidoError::PinRequired);
            }
            Ok(kanso_fido::Registration {
                credential_id: self.key.credential_id().to_vec(),
                auth_data: self.key.registration_auth_data(&request.rp_id),
                client_data_json: kanso_fido::client_data_json(
                    "webauthn.create",
                    &request.challenge,
                    &request.origin,
                ),
                transports: vec!["usb".into()],
            })
        }

        fn assert(
            &self,
            _device: &SecurityKeyDevice,
            request: &kanso_fido::AssertionRequest,
        ) -> Result<kanso_fido::Assertion, FidoError> {
            self.asserts.lock().unwrap().push(request.clone());
            if self.needs_pin && request.pin.is_none() {
                return Err(FidoError::PinRequired);
            }
            let signer = self.forged.as_ref().unwrap_or(&self.key);
            let made = signer.assertion(
                &request.rp_id,
                &request.origin,
                &request.challenge,
                request.hmac_secret,
            );
            Ok(kanso_fido::Assertion {
                credential_id: self.key.credential_id().to_vec(),
                authenticator_data: made.authenticator_data.clone(),
                client_data_json: made.client_data_json.clone(),
                signature: made.signature.clone(),
                user_handle: None,
                hmac_secret: made.prf,
            })
        }
    }

    struct World {
        server: Arc<MemoryServer>,
        secrets: MemorySecrets,
        device: kanso_sync::DeviceIdentity,
    }

    impl World {
        fn on(server: Arc<MemoryServer>) -> World {
            World {
                server,
                secrets: MemorySecrets::default(),
                device: kanso_sync::DeviceIdentity::from_parts(
                    &format!("l_{}", "5".repeat(32)),
                    "This PC",
                    std::array::from_fn(|_| fastrand::u8(..)),
                ),
            }
        }

        fn context(&self) -> PasskeyContext<'_> {
            PasskeyContext {
                accounts: &SyncPasskeyAccounts,
                backend: self.server.as_ref(),
                secrets: &self.secrets,
                server: SERVER,
                device: &self.device,
                rule: RpRule::ServerHost,
                progress: &|_| {},
            }
        }
    }

    #[test]
    fn linux_offers_security_keys_for_its_own_server_only() {
        let server = "https://sync.example.com";
        let advertised = methods("sync.example.com", true);
        assert!(offers_security_keys(
            Platform::Linux,
            Some(&advertised),
            server
        ));
        assert!(!offers_security_keys(
            Platform::Linux,
            Some(&methods("sync.example.com", false)),
            server
        ));
        assert!(!offers_security_keys(
            Platform::Linux,
            Some(&methods("sync.kansoapp.app", true)),
            server
        ));
        assert!(!offers_security_keys(Platform::Linux, None, server));
        for platform in [Platform::MacOs, Platform::Windows] {
            assert!(!offers_security_keys(platform, Some(&advertised), server));
        }
    }

    #[test]
    fn the_key_that_can_unlock_is_picked_first() {
        assert_eq!(pick(&[]), None);
        let keys = [device(false), device(true)];
        assert!(pick(&keys).unwrap().hmac_secret);
        assert!(!pick(&keys[..1]).unwrap().hmac_secret);
    }

    #[test]
    fn a_security_key_signs_up_with_two_touches_and_unlocks_a_new_device() {
        let world = World::on(Arc::new(MemoryServer::new()));
        let fake = FakeKey::new(TestAuthenticator::new(11));
        let key = FidoKey::new(&fake, device(true), None);
        let created =
            passkey_accounts::sign_up(&world.context(), "quiet-otter-4821", &key.passkeys())
                .unwrap();
        assert!(created.can_unlock, "the verified second touch gave a PRF");
        // The make-credential enabled hmac-secret; the second touch was a
        // local assertion for the new credential, never the server's
        // challenge.
        assert!(fake.registers.lock().unwrap()[0].hmac_secret);
        {
            let asserts = fake.asserts.lock().unwrap();
            assert_eq!(asserts.len(), 1);
            assert_eq!(
                asserts[0].allow_credentials,
                vec![fake.key.credential_id().to_vec()]
            );
            assert!(asserts[0].hmac_secret);
            assert_ne!(
                asserts[0].challenge,
                fake.registers.lock().unwrap()[0].challenge
            );
        }

        // Username-first on another device: the allowList names the key's
        // credential and the PRF opens the wrap.
        let elsewhere = World::on(world.server.clone());
        let signed = passkey_accounts::sign_in(
            &elsewhere.context(),
            Some("quiet-otter-4821"),
            &key.passkeys(),
        )
        .unwrap();
        assert!(signed.prf);
        assert!(matches!(signed.signed, PasskeySignIn::Unlocked(_)));
        let asserts = fake.asserts.lock().unwrap();
        assert_eq!(
            asserts.last().unwrap().allow_credentials,
            vec![fake.key.credential_id().to_vec()]
        );
    }

    #[test]
    fn a_second_touch_that_doesnt_verify_registers_without_a_wrap() {
        let world = World::on(Arc::new(MemoryServer::new()));
        let mut fake = FakeKey::new(TestAuthenticator::new(12));
        fake.forged = Some(TestAuthenticator::new(13));
        let key = FidoKey::new(&fake, device(true), None);
        let created =
            passkey_accounts::sign_up(&world.context(), "quiet-otter-4821", &key.passkeys())
                .unwrap();
        assert!(!created.can_unlock);
    }

    #[test]
    fn a_key_without_hmac_secret_signs_up_with_one_touch_and_signs_in_only() {
        let world = World::on(Arc::new(MemoryServer::new()));
        let fake = FakeKey::new(TestAuthenticator::new(14));
        let key = FidoKey::new(&fake, device(false), None);
        let created =
            passkey_accounts::sign_up(&world.context(), "quiet-otter-4821", &key.passkeys())
                .unwrap();
        assert!(!created.can_unlock);
        assert!(!fake.registers.lock().unwrap()[0].hmac_secret);
        assert!(fake.asserts.lock().unwrap().is_empty());
        let elsewhere = World::on(world.server.clone());
        let signed = passkey_accounts::sign_in(
            &elsewhere.context(),
            Some("quiet-otter-4821"),
            &key.passkeys(),
        )
        .unwrap();
        assert!(!signed.prf);
        assert!(!fake.asserts.lock().unwrap()[0].hmac_secret);
        assert!(matches!(signed.signed, PasskeySignIn::Locked(_)));
    }

    #[test]
    fn a_key_that_needs_its_pin_says_so_and_takes_it() {
        let mut fake = FakeKey::new(TestAuthenticator::new(15));
        fake.needs_pin = true;
        let challenge = PasskeyAssertionChallenge {
            rp_id: RP_ID.into(),
            origin: SERVER.into(),
            challenge: vec![6; 32],
            allow_credentials: vec![fake.key.credential_id().to_vec()],
            request_prf: true,
        };
        let without = FidoKey::new(&fake, device(true), None);
        assert_eq!(
            without.passkeys().assert(&challenge).err(),
            Some(PasskeyCeremonyError::Failed)
        );
        assert!(without.pin_required());
        // An empty PIN field is no PIN.
        let empty = FidoKey::new(&fake, device(true), Some(Zeroizing::new(String::new())));
        assert!(empty.passkeys().assert(&challenge).is_err());
        assert!(empty.pin_required());
        let with = FidoKey::new(&fake, device(true), Some(Zeroizing::new("4821".into())));
        let assertion = with.passkeys().assert(&challenge).unwrap();
        assert!(!with.pin_required());
        assert!(assertion.prf.is_some());
        let asserts = fake.asserts.lock().unwrap();
        assert_eq!(
            asserts.last().unwrap().pin.as_deref().map(String::as_str),
            Some("4821")
        );
    }

    #[test]
    fn a_missing_key_is_unavailable_and_a_refusal_failed() {
        struct Gone(FidoError);
        impl Fido for Gone {
            fn register(
                &self,
                _: &SecurityKeyDevice,
                _: &kanso_fido::RegistrationRequest,
            ) -> Result<kanso_fido::Registration, FidoError> {
                Err(self.0.clone())
            }
            fn assert(
                &self,
                _: &SecurityKeyDevice,
                _: &kanso_fido::AssertionRequest,
            ) -> Result<kanso_fido::Assertion, FidoError> {
                Err(self.0.clone())
            }
        }
        let challenge = PasskeyRegistrationChallenge {
            rp_id: RP_ID.into(),
            origin: SERVER.into(),
            challenge: vec![3; 32],
            user_handle: vec![5; 16],
            username: "quiet-otter-4821".into(),
            display_name: "quiet-otter-4821".into(),
            exclude_credentials: vec![],
            request_prf: true,
        };
        let gone = Gone(FidoError::NoDevice);
        let key = FidoKey::new(&gone, device(true), None);
        assert_eq!(
            key.passkeys().register(&challenge).err(),
            Some(PasskeyCeremonyError::Unavailable)
        );
        let refused = Gone(FidoError::Refused("timeout".into()));
        let key = FidoKey::new(&refused, device(true), None);
        assert_eq!(
            key.passkeys().register(&challenge).err(),
            Some(PasskeyCeremonyError::Failed)
        );
        assert!(!key.pin_required());
    }
}
