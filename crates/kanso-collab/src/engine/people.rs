//! The people an account shares with (docs/ACCOUNTS.md › Collaboration ›
//! Identity keys): looked up by username, their keys pinned to their
//! account the first time this library sees them, verified by comparing
//! fingerprints, and refused when the server later gives other keys.

use kanso_core::now_ms;
use kanso_store::Contact;

use super::Engine;
use crate::api::{CollabError, Result};
use crate::codec::b64;
use crate::identity::{Verified, check_public, fingerprint};
use crate::model::{Member, Person, PublicIdentity};

/// What pinning a public identity found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Pinned {
    /// First seen now, or the same keys as pinned.
    Known(Verified),
    /// The keys differ from the pinned ones.
    Changed { pinned: Contact, offered: Verified },
}

impl Engine {
    /// Checks a public identity and pins it, the first time; refuses keys
    /// that differ from those pinned.
    pub(crate) fn pin(&self, identity: &PublicIdentity) -> Result<Pinned> {
        let verified = check_public(identity)?;
        let now = now_ms();
        match self.store.contact(&verified.account_id)? {
            None => {
                self.store.put_contact(&Contact {
                    account_id: verified.account_id.clone(),
                    username: verified.username.clone(),
                    sign_pub: b64(&verified.sign_pub),
                    box_pub: b64(&verified.box_pub),
                    verified_at: None,
                    seen_at: now,
                })?;
                Ok(Pinned::Known(verified))
            }
            Some(contact)
                if contact.sign_pub == b64(&verified.sign_pub)
                    && contact.box_pub == b64(&verified.box_pub) =>
            {
                if contact.username != verified.username {
                    self.store.put_contact(&Contact {
                        username: verified.username.clone(),
                        seen_at: now,
                        ..contact
                    })?;
                }
                Ok(Pinned::Known(verified))
            }
            Some(pinned) => Ok(Pinned::Changed {
                pinned,
                offered: verified,
            }),
        }
    }

    /// Pins a member's keys as a list the owner signed gives them.
    pub(crate) fn pin_member(&self, member: &Member) -> Result<Verified> {
        let sign_pub = crate::codec::unb64_array::<32>(&member.sign_pub)
            .ok_or_else(|| CollabError::Crypto("a member's key isn't well-formed".into()))?;
        let box_pub = crate::codec::unb64_array::<32>(&member.box_pub)
            .ok_or_else(|| CollabError::Crypto("a member's key isn't well-formed".into()))?;
        if member.account == self.account.caller.account_id {
            return Ok(Verified {
                account_id: member.account.clone(),
                username: member.username.clone(),
                sign_pub,
                box_pub,
            });
        }
        match self.store.contact(&member.account)? {
            Some(contact)
                if contact.sign_pub != member.sign_pub || contact.box_pub != member.box_pub =>
            {
                Err(CollabError::KeyChanged(member.username.clone()))
            }
            Some(_) => Ok(Verified {
                account_id: member.account.clone(),
                username: member.username.clone(),
                sign_pub,
                box_pub,
            }),
            // First seen: the keys must be the ones the account published,
            // its binding checked, never just what a list says.
            None => {
                let published = self
                    .api
                    .lookup(&self.account.caller, &member.username)?
                    .ok_or_else(|| CollabError::NoSuchUser(member.username.clone()))?;
                let verified = check_public(&published)?;
                if verified.account_id != member.account
                    || verified.sign_pub != sign_pub
                    || verified.box_pub != box_pub
                {
                    return Err(CollabError::Crypto(format!(
                        "{}'s keys in the note's list aren't the ones their account published",
                        member.username
                    )));
                }
                match self.pin(&published)? {
                    Pinned::Known(verified) => Ok(verified),
                    Pinned::Changed { offered, .. } => {
                        Err(CollabError::KeyChanged(offered.username))
                    }
                }
            }
        }
    }

    /// Someone to share with, by username: their keys checked and pinned.
    pub fn person(&self, username: &str) -> Result<Person> {
        let username = kanso_sync::normalize_username(username)
            .map_err(|_| CollabError::NoSuchUser(username.trim().to_owned()))?;
        if username == self.account.caller.username {
            return Err(CollabError::Refused("That's you.".into()));
        }
        let identity = self
            .api
            .lookup(&self.account.caller, &username)?
            .ok_or_else(|| CollabError::NoSuchUser(username.clone()))?;
        match self.pin(&identity)? {
            Pinned::Known(verified) => self.person_of(&verified.account_id),
            Pinned::Changed { offered, .. } => Err(CollabError::KeyChanged(offered.username)),
        }
    }

    /// A pinned contact as a [`Person`].
    pub fn person_of(&self, account_id: &str) -> Result<Person> {
        if account_id == self.account.caller.account_id {
            let identity = self.identity()?;
            return Ok(Person {
                account_id: account_id.to_owned(),
                username: self.account.caller.username.clone(),
                fingerprint: identity.fingerprint(),
                verified: true,
            });
        }
        let contact = self
            .store
            .contact(account_id)?
            .ok_or_else(|| CollabError::Refused("That person isn't known here.".into()))?;
        let sign = crate::codec::unb64_array::<32>(&contact.sign_pub).unwrap_or([0; 32]);
        let boxed = crate::codec::unb64_array::<32>(&contact.box_pub).unwrap_or([0; 32]);
        Ok(Person {
            account_id: contact.account_id,
            username: contact.username,
            fingerprint: fingerprint(&sign, &boxed),
            verified: contact.verified_at.is_some(),
        })
    }

    /// Marks a contact's fingerprint as compared and matching.
    pub fn verify_person(&self, account_id: &str) -> Result<()> {
        let contact = self
            .store
            .contact(account_id)?
            .ok_or_else(|| CollabError::Refused("That person isn't known here.".into()))?;
        self.store.put_contact(&Contact {
            verified_at: Some(now_ms()),
            ..contact
        })?;
        Ok(())
    }

    /// The fingerprint of the keys the server now gives for someone whose
    /// key changed, to compare before [`Engine::accept_new_key`].
    pub fn offered_key(&self, username: &str) -> Result<Person> {
        let identity = self
            .api
            .lookup(&self.account.caller, username)?
            .ok_or_else(|| CollabError::NoSuchUser(username.to_owned()))?;
        let verified = check_public(&identity)?;
        Ok(Person {
            account_id: verified.account_id.clone(),
            username: verified.username.clone(),
            fingerprint: verified.fingerprint(),
            verified: false,
        })
    }

    /// Pins the keys the server now gives for someone, after the user
    /// compared the new fingerprint: `fingerprint` is the one they saw
    /// ([`Engine::offered_key`]), and keys with another are refused. They
    /// start unverified.
    pub fn accept_new_key(&self, username: &str, fingerprint: &str) -> Result<Person> {
        let identity = self
            .api
            .lookup(&self.account.caller, username)?
            .ok_or_else(|| CollabError::NoSuchUser(username.to_owned()))?;
        let verified = check_public(&identity)?;
        if verified.fingerprint() != fingerprint {
            return Err(CollabError::KeyChanged(verified.username));
        }
        self.store.put_contact(&Contact {
            account_id: verified.account_id.clone(),
            username: verified.username.clone(),
            sign_pub: b64(&verified.sign_pub),
            box_pub: b64(&verified.box_pub),
            verified_at: None,
            seen_at: now_ms(),
        })?;
        self.person_of(&verified.account_id)
    }

    /// A pinned contact's keys, for wrapping to them.
    pub(crate) fn pinned(&self, account_id: &str) -> Result<Verified> {
        let contact = self
            .store
            .contact(account_id)?
            .ok_or_else(|| CollabError::Refused("That person isn't known here.".into()))?;
        Ok(Verified {
            account_id: contact.account_id,
            username: contact.username,
            sign_pub: crate::codec::unb64_array::<32>(&contact.sign_pub)
                .ok_or_else(|| CollabError::Crypto("a pinned key isn't well-formed".into()))?,
            box_pub: crate::codec::unb64_array::<32>(&contact.box_pub)
                .ok_or_else(|| CollabError::Crypto("a pinned key isn't well-formed".into()))?,
        })
    }
}
